use bytes::BytesMut;
use tokio::io::{self, AsyncWriteExt, AsyncReadExt};
use tokio::net::TcpStream;
use tokio::time::{timeout, Instant};
use std::env;
use std::error::Error;
use std::future::Future;
use std::time::Duration;
use bytes::Buf;

use crate::config::{ Config, load_config };
use crate::llrp::{get_message_type_str, LlrpMessage, LlrpMessageType, LlrpResponse};

pub struct LlrpClient {
  stream      : TcpStream,
  message_id  : u32,
  config      : Config
}

impl LlrpClient {

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
    
    Ok(LlrpClient { stream, message_id: 1001, config })
  }

  pub async fn disconnect(
    &mut self, 
  ) -> Result<(), Box<dyn Error>> {

    self.send_close_connection().await?;

    Ok(())
  }

  pub fn next_message_id(
    &mut self
  ) -> u32 {

    let current_id = self.message_id;
    self.message_id += 1;
    
    current_id
  }

  async fn send_close_connection(
    &mut self, 
  ) -> Result<(), Box<dyn Error>> {

    let message_id = self.next_message_id();

    let message = LlrpMessage::new(LlrpMessageType::CloseConnection, message_id, vec![]);
    self.stream.write_all(&message.encode()).await?;

    if self.config.await_response_ack {
      let response = self.receive_response().await?;
      self.log_response_acknowledgment(LlrpMessageType::CloseConnectionResponse, response.message_type);
    }
    
    Ok(())
  }

  pub async fn send_keep_alive(
    &mut self, 
  ) -> Result<(), Box<dyn Error>> {

    let message_id = self.next_message_id();

    let message = LlrpMessage::new(LlrpMessageType::Keepalive, message_id, vec![]);
    self.stream.write_all(&message.encode()).await?;
    
    if self.config.await_response_ack {
      let response = self.receive_response().await?;
      self.log_response_acknowledgment(LlrpMessageType::KeepaliveAck, response.message_type);
    }

    Ok(())
  }

  pub async fn send_enable_events_and_reports(
    &mut self, 
  ) -> Result<(), Box<dyn Error>> {

    let message_id = self.next_message_id();
    
    let message = LlrpMessage::new_enable_events_and_reports(message_id);
    self.stream.write_all(&message.encode()).await?;
    
    if self.config.await_response_ack {
      let response = self.receive_response().await?;
      self.log_response_acknowledgment(LlrpMessageType::EnableEventsAndReports, response.message_type);
    }

    Ok(())
  }

  pub async fn send_get_reader_capabilities(
    &mut self,
  ) -> Result<(), Box<dyn Error>> {

    let message_id = self.next_message_id();

    let message = LlrpMessage::new_get_reader_capabilities(message_id);
    self.stream.write_all(&message.encode()).await?;

    let response = self.receive_response().await?;
    if response.message_type == LlrpMessageType::SetReaderConfigResponse {
      response.decode_reader_capabilities()?;
    }

    Ok(())
  }

  pub async fn send_get_reader_config(
    &mut self,
  ) -> Result<(), Box<dyn Error>> {

    let message_id = self.next_message_id();

    let message = LlrpMessage::new_get_reader_config(message_id);
    self.stream.write_all(&message.encode()).await?;

    if self.config.await_response_ack {
      let response = self.receive_response().await?;
      self.log_response_acknowledgment(LlrpMessageType::GetReaderConfigResponse, response.message_type);
    }

    Ok(())
  }

  pub async fn send_set_reader_config(
    &mut self, 
  ) -> Result<(), Box<dyn Error>> {
    
    let message_id = self.next_message_id();

    let message = LlrpMessage::new_set_reader_config(message_id);
    self.stream.write_all(&message.encode()).await?;

    if self.config.await_response_ack {
      let response = self.receive_response().await?;
      self.log_response_acknowledgment(LlrpMessageType::SetReaderConfigResponse, response.message_type);
    }

    Ok(())
  }

  pub async fn send_add_rospec(
    &mut self,
  ) -> Result<(), Box<dyn Error>> {
    
    let message_id = self.next_message_id();
    let message = LlrpMessage::new_add_rospec(message_id, &self.config.ROSpec);

    self.stream.write_all(&message.encode()).await?;
    
    if self.config.await_response_ack {
      let response = self.receive_response().await?;
      self.log_response_acknowledgment(LlrpMessageType::AddROspecResponse, response.message_type);
    }

    Ok(())
  }

  pub async fn send_enable_rospec(
    &mut self, 
  ) -> Result<(), Box<dyn Error>> {
    
    let message_id = self.next_message_id();

    let message = LlrpMessage::new_enable_rospec(message_id, self.config.ROSpec.rospec_id);
    self.stream.write_all(&message.encode()).await?;

    if self.config.await_response_ack {
      let response = self.receive_response().await?;
      self.log_response_acknowledgment(LlrpMessageType::EnableROspecResponse, response.message_type);
    }

    Ok(())
  }

  pub async fn send_start_rospec(
    &mut self, 
  ) -> Result<(), Box<dyn Error>> {

    let message_id = self.next_message_id();

    let message = LlrpMessage::new_start_rospec(message_id, self.config.ROSpec.rospec_id);
    self.stream.write_all(&message.encode()).await?;

    if self.config.await_response_ack {
      let response = self.receive_response().await?;
      self.log_response_acknowledgment(LlrpMessageType::StartROspecResponse, response.message_type);
    }

    Ok(())
  }

  pub async fn send_stop_rospec(
    &mut self, 
  ) -> Result<(), Box<dyn Error>> {

    let message_id = self.next_message_id();

    let message = LlrpMessage::new_stop_rospec(message_id, self.config.ROSpec.rospec_id);
    self.stream.write_all(&message.encode()).await?;

    if self.config.await_response_ack {
      let response = self.receive_response().await?;
      self.log_response_acknowledgment(LlrpMessageType::StopROspecResponse, response.message_type);
    }

    Ok(())
  }

  pub async fn send_delete_rospec(
    &mut self,
    rospec_id: u32
  ) -> Result<(), Box<dyn Error>> {

    let message_id = self.next_message_id();

    let message = LlrpMessage::new_delete_rospec(message_id, rospec_id);
    self.stream.write_all(&message.encode()).await?;

    if self.config.await_response_ack {
      let response = self.receive_response().await?;
      self.log_response_acknowledgment(LlrpMessageType::DeleteROspecResponse, response.message_type);
    }

    Ok(())
  }

  fn log_response_acknowledgment(
    &mut self, 
    expected_response_type : LlrpMessageType, 
    response_type          : LlrpMessageType
  ) {

    if expected_response_type != expected_response_type {
      println!("[Warning] Expected {:?} Acknowledgment, received {} instead", get_message_type_str(expected_response_type.value()), get_message_type_str(response_type.value()));
    } else {
      println!("[ACK] {}", get_message_type_str(expected_response_type.value()));
    }
  }

  pub async fn await_ro_access_report<Fut, F>(
    &mut self,
    mut response_callback : F
  ) -> Result<(), Box<dyn Error>> 
  where
    F   : FnMut(LlrpResponse) -> Fut + Send + Sync,
    Fut : Future<Output = ()> + Send 
  {

    let _timeout = Duration::from_millis(self.config.ro_access_report_timeout);
    let start_time = Instant::now();

    loop {

      let elapsed = start_time.elapsed();
      if elapsed >= _timeout {
        return Err(Box::new(std::io::Error::new(
          std::io::ErrorKind::TimedOut,
          "Timeout waiting for ROAccessReport",
        )));
      }

      match self.receive_response().await {

        Ok(response) => {
          if response.message_type == LlrpMessageType::ROAccessReport {
            response_callback(response).await;
            break;
          } else {
            continue;
          }
        }

        Err(e) => {
          if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
            if io_err.kind() == std::io::ErrorKind::TimedOut {
              println!("Timeout while waiting for ROAccessReport, retrying...");
              continue;
            } else {
              return Err(e);
            }
          } else {
            return Err(e);
          }
        }
      }
    }

    Ok(())
  }

  async fn receive_response(
    &mut self
  ) -> Result<LlrpResponse, Box<dyn Error>> {

    let mut buf = BytesMut::with_capacity(1024);
    let result = timeout(Duration::from_millis(self.config.response_timeout), async {

      while buf.len() < 10 {
        let n = self.stream.read_buf(&mut buf).await?;
        if n == 0 {
          return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "Connection closed",
          )));
        }
      }

      let mut header_buf = buf.clone();
      let version_type = header_buf.get_u16();
      let version = (version_type >> 13) & 0x7;
      let message_type = version_type & 0x3FF;
      let message_length = header_buf.get_u32();
      let message_id = header_buf.get_u32();

      if message_length < 10 {
        return Err(Box::new(std::io::Error::new(
          std::io::ErrorKind::InvalidData,
          "Invalid message length in header",
        )));
      }

      while buf.len() < message_length as usize {
        let n = self.stream.read_buf(&mut buf).await?;
        if n == 0 {
          return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "Connection closed",
          )));
        }
      }

      Ok(())
    }).await;

    if let Err(_) = result {
      return Err(Box::new(std::io::Error::new(
        std::io::ErrorKind::TimedOut,
        "Timeout while waiting for response",
      )));
    }

    let llrp_message = LlrpMessage::decode(&mut buf)?;
    let llrp_response = LlrpResponse::from_message(llrp_message);

    Ok(llrp_response)
  }

}