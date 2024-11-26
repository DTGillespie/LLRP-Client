use bytes::BytesMut;
use tokio::io::{self, AsyncWriteExt, AsyncReadExt};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, oneshot, Mutex};
use tokio::time::{timeout, Instant};
use std::collections::HashMap;
use std::error::Error;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use bytes::Buf;

use crate::client;
use crate::config::{ Config, load_config };
use crate::llrp::{get_message_type_str, LlrpMessage, LlrpMessageType, LlrpResponse};

pub struct LlrpClient {
  stream            : Arc<Mutex<TcpStream>>,
  message_id        : u32,
  config            : Config,
  response_handlers : Arc<Mutex<HashMap<u32, oneshot::Sender<LlrpResponse>>>>,
  ro_report_tx      : broadcast::Sender<LlrpResponse>
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
      eprintln!("Error loading LLRP configuration: {}", e);
      io::Error::new(
        io::ErrorKind::InvalidInput,
        "Failed to load LLRP configuration. Please verify the configuration file path and content."
      )
    })?;

    let stream = TcpStream::connect(&config.host).await.map_err(|e| {
      eprintln!("Error connecting to LLRP server at {}: {}", config.host, e);
      io::Error::new(
        io::ErrorKind::ConnectionRefused,
        "Unable to connect to LLRP server"
      )
    })?;

    println!("Client Successfully Connected to LLRP server: {}", config.host);
    
    let stream = Arc::new(Mutex::new(stream));

    let (ro_report_tx, _) = broadcast::channel(100);

    let client = LlrpClient { 
      stream: stream.clone(), 
      message_id: 1001, 
      config,
      response_handlers: Arc::new(Mutex::new(HashMap::new())),
      ro_report_tx
    };

    let stream_clone = stream.clone();
    let response_handler_clone = client.response_handlers.clone();
    let ro_report_tx_clone = client.ro_report_tx.clone();

    tokio::spawn(async move {
      if let Err(e) = LlrpClient::receive_loop(
        stream_clone, 
        response_handler_clone,
        ro_report_tx_clone
      ).await {
        eprintln!("Error in response handler loop: {}", e);
      }
    });

    Ok(client)
  }

  async fn send_message(
    &mut self,
    message: LlrpMessage
  ) -> Result<LlrpResponse, Box<dyn Error>> {

    let message_id = message.message_id;
    let (sender, receiver) = oneshot::channel();

    {
      let mut handlers = self.response_handlers.lock().await;
      handlers.insert(message_id, sender);
    }

    {
      let mut stream = self.stream.lock().await;
      stream.write_all(&message.encode()).await?;

    }
    
    let timeout_duration = Duration::from_millis(self.config.response_timeout);
    match timeout(timeout_duration, receiver).await {
      
      Ok(response) => response.map_err(|_| {
        Box::new(io::Error::new(
          io::ErrorKind::Other,
          "Rersponse handler dropped"
        )) as Box<dyn Error>
      }),

      Err(_) => {
        //let mut handlers = self.response_handlers.lock().await;
        //handlers.remove(&message_id);
        Err(Box::new(io::Error::new(
          io::ErrorKind::TimedOut,
          "Timeout while waiting for response"
        )))
      }
    }
  }

  async fn send_message_ack(
    &mut self,
    message                : LlrpMessage,
    expected_response_type : LlrpMessageType
  ) -> Result<(), Box<dyn Error>> {

    let response = self.send_message(message).await?;
    if self.config.log_response_ack {
      self.log_response_acknowledgment(expected_response_type, response.message_type);
    }

    Ok(())
  }

  pub async fn send_close_connection(
    &mut self, 
  ) -> Result<(), Box<dyn Error>> {

    let message_id = self.next_message_id();

    let message = LlrpMessage::new(LlrpMessageType::CloseConnection, message_id, vec![]);
    self.send_message_ack(message, LlrpMessageType::CloseConnectionResponse)
      .await    
  }

  pub async fn send_keep_alive(
    &mut self, 
  ) -> Result<(), Box<dyn Error>> {

    let message_id = self.next_message_id();

    let message = LlrpMessage::new(LlrpMessageType::Keepalive, message_id, vec![]);
    self.send_message_ack(message, LlrpMessageType::KeepaliveAck)
      .await
  }

  pub async fn send_enable_events_and_reports(
    &mut self, 
  ) -> Result<(), Box<dyn Error>> {

    let message_id = self.next_message_id();
    
    let message = LlrpMessage::new_enable_events_and_reports(message_id);
    self.send_message(message).await?;
    
    Ok(())
  }

  // Custom response decoding & handling
  pub async fn send_get_reader_capabilities(
    &mut self,
  ) -> Result<(), Box<dyn Error>> {

    let message_id = self.next_message_id();

    let message = LlrpMessage::new_get_reader_capabilities(message_id);
    let response = self.send_message(message).await?;
    
    if response.message_type == LlrpMessageType::GetReaderCapabilitiesResponse {
      response.decode_reader_capabilities()?;

      if self.config.log_response_ack {
        self.log_response_acknowledgment(
          LlrpMessageType::GetReaderCapabilitiesResponse, 
          response.message_type
        );
      }

    } else {
      return Err(Box::new(io::Error::new(
        io::ErrorKind::InvalidData,
        format!(
          "Expected GetReaderCapabilitiesResppnse, received: {:?}",
          response.message_type
        )
      )));
    }

    Ok(())
  }

  // Custom response decoding & handling
  pub async fn send_get_reader_config(
    &mut self,
  ) -> Result<(), Box<dyn Error>> {

    let message_id = self.next_message_id();

    let message = LlrpMessage::new_get_reader_config(message_id);
    let response = self.send_message(message).await?;

    if response.message_type == LlrpMessageType::GetReaderConfigResponse {
      response.decode_reader_config()?;

      if self.config.log_response_ack {
        self.log_response_acknowledgment(
          LlrpMessageType::GetReaderConfigResponse, 
          response.message_type
        );
      }

    } else {
      return Err(Box::new(io::Error::new(
        io::ErrorKind::InvalidData,
        format!(
          "Expected GetReaderConfigResppnse, received: {:?}",
          response.message_type
        )
      )));
    }

    Ok(())
  }

  pub async fn send_set_reader_config(
    &mut self, 
  ) -> Result<(), Box<dyn Error>> {
    
    let message_id = self.next_message_id();
    
    let message = LlrpMessage::new_set_reader_config(message_id);
    self.send_message_ack(message, LlrpMessageType::SetReaderConfigResponse)
      .await
  }

  pub async fn send_add_rospec(
    &mut self,
  ) -> Result<(), Box<dyn Error>> {
    
    let message_id = self.next_message_id();
    
    let message = LlrpMessage::new_add_rospec(message_id, &self.config.ROSpec);
    self.send_message_ack(message, LlrpMessageType::AddROspecResponse)
      .await
  }

  pub async fn send_enable_rospec(
    &mut self, 
  ) -> Result<(), Box<dyn Error>> {
    
    let message_id = self.next_message_id();

    let message = LlrpMessage::new_enable_rospec(message_id, self.config.ROSpec.rospec_id);
    self.send_message_ack(message, LlrpMessageType::EnableROspecResponse)
      .await
  }

  pub async fn send_start_rospec(
    &mut self, 
  ) -> Result<(), Box<dyn Error>> {

    let message_id = self.next_message_id();

    let message = LlrpMessage::new_start_rospec(message_id, self.config.ROSpec.rospec_id);
    self.send_message_ack(message, LlrpMessageType::StartROspecResponse)
      .await
  }

  pub async fn send_stop_rospec(
    &mut self, 
  ) -> Result<(), Box<dyn Error>> {

    let message_id = self.next_message_id();

    let message = LlrpMessage::new_stop_rospec(message_id, self.config.ROSpec.rospec_id);
    self.send_message_ack(message, LlrpMessageType::StartROspecResponse)
      .await
  }

  pub async fn send_delete_rospec(
    &mut self,
    rospec_id: u32
  ) -> Result<(), Box<dyn Error>> {

    let message_id = self.next_message_id();

    let message = LlrpMessage::new_delete_rospec(message_id, rospec_id);
    self.send_message_ack(message, LlrpMessageType::DeleteROspecResponse)
      .await
  }

  pub async fn await_ro_access_report<Fut, F>(
    &mut self,
    mut response_callback : F
  ) -> Result<(), Box<dyn Error>> 
  where
    F   : FnMut(LlrpResponse) -> Fut + Send + Sync,
    Fut : Future<Output = ()> + Send 
  {

    let mut ro_report_rx = self.ro_report_tx.subscribe();

    let _timeout = Duration::from_millis(self.config.response_timeout);
    let start_time = Instant::now();

    loop {

      let elapsed = start_time.elapsed();
      if elapsed >= _timeout {
        return Err(Box::new(std::io::Error::new(
          std::io::ErrorKind::TimedOut,
          "Timeout waiting for ROAccessReport",
        )));
      }

      match ro_report_rx.recv().await {

        Ok(response) => {
          response_callback(response).await;
          break;
        }

        Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
          eprintln!("Skipped {} messages due to buffer overflow", skipped);
          continue;
        }

        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
          return Err(Box::new(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "ROAccessReport channel closed"
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
      println!("[ACK] {}", get_message_type_str(expected_response_type.value()));
    }
  }

  async fn receive_loop(
    stream        : Arc<Mutex<TcpStream>>,
    response_handlers : Arc<Mutex<HashMap<u32, oneshot::Sender<LlrpResponse>>>>,
    ro_report_tx      : broadcast::Sender<LlrpResponse>
  ) -> Result<(), Box<dyn Error>> {
    
    let mut buf = BytesMut::with_capacity(1024);

    loop {
      {

        let mut stream = stream.lock().await;

        while buf.len() < 10 {
          let n = stream.read_buf(&mut buf).await?;
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

        let mut stream = stream.lock().await;
        
        let n = stream.read_buf(&mut buf).await?;
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

        _ => {

          let mut handlers = response_handlers.lock().await;
          
          if let Some(sender) = handlers.remove(&llrp_response.message_id) {
            let _ = sender.send(llrp_response);
          } else {
            eprintln!("Received response with unknown message_id: {}", llrp_response.message_id);
          }
        }
      }
    }
  }

}