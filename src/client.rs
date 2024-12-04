use bytes::BytesMut;
use tokio::io::{self, split, AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, Mutex};
use tokio::time::{timeout, Instant};
use std::error::Error;
use std::future::Future;
use std::sync::{Arc, Once};
use std::time::Duration;
use bytes::Buf;
use env_logger::{self, Builder};
use std::fs::OpenOptions;
use chrono::Local;
use std::io::Write;
use log::{info, debug, warn, error, LevelFilter};
use std::collections::HashMap;

use crate::config::{ Config, load_config };
use crate::llrp::{get_message_type_str, LlrpMessage, LlrpMessageType, LlrpResponse, LlrpResponseData};

static INIT_LOGGER: Once = Once::new();

pub struct LlrpClient {
  reader            : Arc<Mutex<ReadHalf<TcpStream>>>,
  writer            : Arc<Mutex<WriteHalf<TcpStream>>>,
  message_id        : u32,
  config            : Config,
  message_tx        : broadcast::Sender<LlrpResponse>,
  ro_report_tx      : broadcast::Sender<LlrpResponse>
}

fn configure_logger(log_level: &str) {
  INIT_LOGGER.call_once(|| {

    let file = OpenOptions::new()
      .create(true) // Create file if it does not exist
      .append(true) // Append to file instead of truncating it
      .open("system.log")
      .expect("Failed to open system.log");

    let mut builder = Builder::from_default_env();

    builder.format(move |buf, record| {
      let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
      writeln!(buf, "[{}] {} - {}", timestamp, record.level(), record.args())
    });

    if let Some(level) = parse_log_level(log_level) {
      builder.filter(None, level);
    } else {
      eprintln!("Invalid log level: {}. Defaulting to Debug.", log_level);
      builder.filter(None, LevelFilter::Debug);
    }

    builder.target(env_logger::Target::Pipe(Box::new(file)));
    
    builder.init();
  });
}

fn parse_log_level(level: &str) -> Option<LevelFilter> {
  
  let levels: HashMap<&str, LevelFilter> = HashMap::from([
    ("off", LevelFilter::Off),
    ("error", LevelFilter::Error),
    ("warn", LevelFilter::Warn),
    ("info", LevelFilter::Info),
    ("debug", LevelFilter::Debug),
    ("trace", LevelFilter::Trace),
  ]);

  levels.get(level.to_lowercase().as_str()).cloned()
}

impl LlrpClient {

  fn next_message_id(
    &mut self
  ) -> u32 {

    let current_id = self.message_id;
    self.message_id += 1;
    
    current_id
  }

  pub async fn initialize(
    configuration_path: &str
  ) -> io::Result<Self> {

    let config = load_config(configuration_path).map_err(|e| {
      io::Error::new(
        io::ErrorKind::InvalidInput,
        "Failed to load LLRP configuration. Please verify the configuration file path and content."
      )
    })?;

    configure_logger(config.log_level.as_str());

    let stream = TcpStream::connect(&config.host).await.map_err(|e| {
      error!("Error connecting to LLRP server at {}: {}", config.host, e);
      io::Error::new(
        io::ErrorKind::ConnectionRefused,
        "Unable to connect to LLRP server"
      )
    })?;

    info!("Client Successfully Connected to LLRP server: {}", config.host);
    
    let (reader, writer) = split(stream);
    let (message_tx, _) = broadcast::channel(100);
    let (ro_report_tx, _) = broadcast::channel(100);

    let client_message_tx = message_tx.clone();

    let client = LlrpClient {
      reader: Arc::new(Mutex::new(reader)),
      writer: Arc::new(Mutex::new(writer)),
      message_id: 1001, 
      config,
      message_tx: client_message_tx,
      ro_report_tx
    };

    let reader_clone = client.reader.clone();
    let message_tx_clone = message_tx.clone();
    let ro_report_tx_clone = client.ro_report_tx.clone();

    tokio::spawn(async move {
      if let Err(e) = LlrpClient::receive_loop(
        reader_clone,
        message_tx_clone,
        ro_report_tx_clone
      ).await {
        error!("Error in response handler loop: {}", e);
      }
    });

    Ok(client)
  }

  async fn send_message(
    &mut self,
    message: LlrpMessage,
    expected_response_type : LlrpMessageType
  ) -> Result<LlrpResponse, Box<dyn Error>> {

    {
      let mut writer = self.writer.lock().await;
      writer.write_all(&message.encode()).await?;
    }

    if expected_response_type == LlrpMessageType::None {
      return Ok(LlrpResponse {
        message_type: LlrpMessageType::None,
        message_id: message.message_id,
        payload: vec![]
      });
    }
    
    let mut message_rx = self.message_tx.subscribe();
    let timeout_duration = Duration::from_millis(self.config.response_timeout);
    let start_time = Instant::now();

    loop {

      let elapsed = start_time.elapsed();
      if elapsed >= timeout_duration {
        return Err(Box::new(io::Error::new(
          io::ErrorKind::TimedOut,
          "Timeout while waiting for response"
        )));
      }

      match timeout(timeout_duration - elapsed, message_rx.recv()).await {

        Ok(Ok(llrp_response)) => {
          if llrp_response.message_type == expected_response_type {
            return Ok(llrp_response);
          } else {
            warn!(
              "Received unexpected message type: {:?}",
              llrp_response.message_type
            );
          }
        }

        Ok(Err(broadcast::error::RecvError::Closed)) => {
          return Err(Box::new(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Message channel closed"
          )));
        }

        Ok(Err(broadcast::error::RecvError::Lagged(skipped))) => {
          warn!("Missed {} messages due to buffer overflow", skipped);
        }

        Err(_) => {
          return Err(Box::new(io::Error::new(
            io::ErrorKind::TimedOut,
            "Timeout while waiting for response"
          )));
        }
      }
    }
  }

  async fn send_message_ack(
    &mut self,
    message                : LlrpMessage,
    expected_response_type : LlrpMessageType
  ) -> Result<LlrpResponse, Box<dyn Error>> {

    let response = self.send_message(message, expected_response_type).await?;
    if self.config.log_response_ack && expected_response_type != LlrpMessageType::None {
      self.log_response_acknowledgment(expected_response_type, response.message_type);
    }

    Ok(response)
  }

  pub async fn send_close_connection(
    &mut self, 
  ) -> Result<(), Box<dyn Error>> {

    let message_id = self.next_message_id();

    let message = LlrpMessage::new(LlrpMessageType::CloseConnection, message_id, vec![]);
    let _ = self.send_message_ack(message, LlrpMessageType::CloseConnectionResponse).await;

    Ok(())
  }

  pub async fn send_keep_alive(
    &mut self, 
  ) -> Result<(), Box<dyn Error>> {

    let message_id = self.next_message_id();

    let message = LlrpMessage::new(LlrpMessageType::Keepalive, message_id, vec![]);
    let _ = self.send_message_ack(message, LlrpMessageType::KeepaliveAck).await?;

    Ok(())
  }

  pub async fn send_enable_events_and_reports(
    &mut self, 
  ) -> Result<(), Box<dyn Error>> {

    let message_id = self.next_message_id();
    
    let message = LlrpMessage::new_enable_events_and_reports(message_id);
    let _ = self.send_message_ack(message, LlrpMessageType::None).await?;

    Ok(())
  }

  pub async fn send_get_reader_capabilities<Fut, F>(
    &mut self,
    mut response_callback: F
  ) -> Result<(), Box<dyn Error>> 
  where
    F   : FnMut(LlrpResponseData) -> Fut + Send + Sync,
    Fut : Future<Output = ()> + Send 
  {

    let message_id = self.next_message_id();

    let message = LlrpMessage::new_get_reader_capabilities(message_id);
    let response = self
      .send_message_ack(message, LlrpMessageType::GetReaderCapabilitiesResponse)
      .await?;

    match response.decode() {

      Ok(response_data) => {
        response_callback(response_data).await;
        Ok(())
      }

      Err(e) => Err(Box::new(e))
    }
  }

  pub async fn send_get_reader_config<Fut, F>(
    &mut self,
    mut response_callback: F
  ) -> Result<(), Box<dyn Error>> 
  where
    F   : FnMut(LlrpResponseData) -> Fut + Send + Sync,
    Fut : Future<Output = ()> + Send 
  {

    let message_id = self.next_message_id();

    let message = LlrpMessage::new_get_reader_config(message_id);
    let response = self
      .send_message_ack(message, LlrpMessageType::GetReaderConfigResponse)
      .await?;
    
    match response.decode() {

      Ok(response_data) => {
        response_callback(response_data).await;
        Ok(())
      }

      Err(e) => Err(Box::new(e)),
    }
  }

  pub async fn send_set_reader_config(
    &mut self, 
  ) -> Result<(), Box<dyn Error>> {
    
    let message_id = self.next_message_id();
    
    let message = LlrpMessage::new_set_reader_config(message_id, &self.config.reader_config);
    let _ = self.send_message_ack(message, LlrpMessageType::SetReaderConfigResponse).await?;

    Ok(())
  }

  pub async fn send_add_rospec(
    &mut self,
  ) -> Result<(), Box<dyn Error>> {
    
    let message_id = self.next_message_id();
    
    let message = LlrpMessage::new_add_rospec(message_id, &self.config.rospec);
    let _ = self.send_message_ack(message, LlrpMessageType::AddROspecResponse).await?;

    Ok(())
  }

  pub async fn send_enable_rospec(
    &mut self, 
  ) -> Result<(), Box<dyn Error>> {
    
    let message_id = self.next_message_id();

    let message = LlrpMessage::new_enable_rospec(message_id, self.config.rospec.rospec_id);
    let _ = self.send_message_ack(message, LlrpMessageType::EnableROSpecResponse).await?;

    Ok(())
  }

  pub async fn send_start_rospec(
    &mut self, 
  ) -> Result<(), Box<dyn Error>> {

    let message_id = self.next_message_id();

    let message = LlrpMessage::new_start_rospec(message_id, self.config.rospec.rospec_id);
    let _ = self.send_message_ack(message, LlrpMessageType::StartROSpecResponse).await?;

    Ok(())
  }

  pub async fn send_stop_rospec(
    &mut self, 
  ) -> Result<(), Box<dyn Error>> {

    let message_id = self.next_message_id();

    let message = LlrpMessage::new_stop_rospec(message_id, self.config.rospec.rospec_id);
    let _ = self.send_message_ack(message, LlrpMessageType::StopROSpecResponse).await?;

    Ok(())
  }

  pub async fn send_delete_rospec(
    &mut self,
    rospec_id: u32
  ) -> Result<(), Box<dyn Error>> {

    let message_id = self.next_message_id();

    let message = LlrpMessage::new_delete_rospec(message_id, rospec_id);
    let _ = self.send_message_ack(message, LlrpMessageType::DeleteROSpecResponse).await?;

    Ok(())
  }

  pub async fn await_ro_access_report<Fut, F>(
    &mut self,
    mut response_callback: F
  ) -> Result<(), Box<dyn Error>> 
  where
    F   : FnMut(LlrpResponseData) -> Fut + Send + Sync,
    Fut : Future<Output = ()> + Send 
  {

    let mut ro_report_rx = self.ro_report_tx.subscribe();

    let timeout_duration = Duration::from_millis(self.config.response_timeout);
    let start_time = Instant::now();

    loop {

      let elapsed = start_time.elapsed();
      if elapsed >= timeout_duration {
        return Err(Box::new(std::io::Error::new(
          std::io::ErrorKind::TimedOut,
          "Timeout waiting for ROAccessReport",
        )));
      }

      let remaining_timeout = timeout_duration - elapsed;

      match timeout(remaining_timeout, ro_report_rx.recv()).await {

        Ok(Ok(response)) => {
          match response.decode() {

            Ok(response_data) => {
              response_callback(response_data).await;
              break;
            }

            Err(e) => {
              return Err(Box::new(e));
            }
          }
        }

        Ok(Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped))) => {
          warn!("Skipped {} messages due to buffer overflow", skipped);
          continue;
        }

        Ok(Err(tokio::sync::broadcast::error::RecvError::Closed)) => {
          return Err(Box::new(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "ROAccessReport channel closed"
          )));
        }

        Err(_) => {
          return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "Timeout waiting for ROAccessReport"
          )));
        }
      }
    }

    Ok(())
  }

  fn log_response_acknowledgment(
    &mut self, 
    expected_response_type : LlrpMessageType, 
    response_type          : LlrpMessageType
  ) {

    if expected_response_type == response_type {
      info!("[ACK] {}", get_message_type_str(expected_response_type.value()));
    }
  }

  async fn receive_loop(
    reader            : Arc<Mutex<ReadHalf<TcpStream>>>,
    message_tx        : broadcast::Sender<LlrpResponse>,
    ro_report_tx      : broadcast::Sender<LlrpResponse>
  ) -> Result<(), Box<dyn Error>> {
    
    let mut buf = BytesMut::with_capacity(1024);

    loop {
      {

        let mut reader = reader.lock().await;
        
        while buf.len() < 10 {
          let n = reader.read_buf(&mut buf).await?;
          if n == 0 {
            return Err(Box::new(io::Error::new(
              io::ErrorKind::UnexpectedEof,
              "Connected closed"
            )));
          }
        }
      }
  
      let mut header_buf = buf.clone();
      let version_type = header_buf.get_u16();
      let _version = (version_type >> 10) & 0x7;
      let _message_type = version_type & 0x3FF;
      let message_length = header_buf.get_u32();
      let message_id = header_buf.get_u32();
  
      if message_length < 10 {
        return Err(Box::new(io::Error::new(
          io::ErrorKind::InvalidData,
          "Invalid message length in header"
        )));
      }
  
      while buf.len() < message_length as usize {

        let mut reader = reader.lock().await;
        
        let n = reader.read_buf(&mut buf).await?;
        if n == 0 {
          return Err(Box::new(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Connection closed"
          )));
        }
      }

      let llrp_message = LlrpMessage::decode(&mut buf)?;
      let llrp_response = LlrpResponse::from_message(llrp_message);

      match llrp_response.message_type {

        LlrpMessageType::ROAccessReport => {
          let _ = ro_report_tx.send(llrp_response);
        }

        LlrpMessageType::ReaderEventNotification => {
          continue;
        }

        _ => {
          let _ = message_tx.send(llrp_response);
        }
      }
    }
  }

}