use bytes::BytesMut;
use tokio::io::{self, AsyncWriteExt, AsyncReadExt};
use tokio::net::TcpStream;
use tokio::time::timeout;
use std::error::Error;
use std::hash::Hash;
use std::time::Duration;
use bytes::Buf;

use crate::llrp::{get_message_type_str, LlrpMessage, LlrpMessageType, LlrpResponse};
/*
use crate::llrp::{
  TYPE_KEEPALIVE, 
  TYPE_CLOSE_CONNECTION,
};
*/

pub struct LlrpClient {
  stream: TcpStream,
  message_id: u32,
  response_timeout: u64,
}

impl LlrpClient {

  pub async fn connect(addr: &str, response_timeout: u64) -> io::Result<Self> {
    let stream = TcpStream::connect(addr).await?;
    println!("Client connected: {}", addr);
    
    Ok(LlrpClient { stream, message_id: 1001 , response_timeout})
  }

  pub async fn disconnect(&mut self) -> Result<(), Box<dyn Error>> {
    self.send_close_connection().await?;
    Ok(())
  }

  pub fn next_message_id(&mut self) -> u32 {
    let current_id = self.message_id;
    self.message_id += 1;
    
    current_id
  }

  async fn send_close_connection(&mut self) -> Result<(), Box<dyn Error>> {
    let message_id = self.next_message_id();

    let message = LlrpMessage::new(LlrpMessageType::CloseConnection, message_id, vec![]);
    self.stream.write_all(&message.encode()).await?;

    println!("Sent CLOSE_CONNECTION");

    Ok(())
  }

  pub async fn send_keep_alive(&mut self) -> Result<(), Box<dyn Error>> {
    let message_id = self.next_message_id();

    let message = LlrpMessage::new(LlrpMessageType::Keepalive, message_id, vec![]);
    self.stream.write_all(&message.encode()).await?;
    
    println!("Sent KEEP_ALIVE");

    Ok(())
  }

  pub async fn send_enable_events_and_reports(&mut self) -> Result<(), Box<dyn Error>> {
    let message_id = self.next_message_id();
    
    let message = LlrpMessage::new_enable_events_and_reports(message_id);
    self.stream.write_all(&message.encode()).await?;
    
    println!("Sent ENABLE_EVENTS_AND_REPORTS");
    
    Ok(())
  }

  pub async fn send_add_rospec(&mut self, rospec_id: u32) -> Result<(), Box<dyn Error>> {
    let message_id = self.next_message_id();
    
    let message = LlrpMessage::new_add_rospec(message_id, rospec_id);
    self.stream.write_all(&message.encode()).await?;
    
    println!("Sent ADD_ROSPEC with ID: {}", rospec_id);

    Ok(())
  }

  pub async fn send_enable_rospec(&mut self, rospec_id: u32, receive_response: Option<bool>) -> Result<(), Box<dyn Error>> {
    let message_id = self.next_message_id();

    let message = LlrpMessage::new_enable_rospec(message_id, rospec_id);
    self.stream.write_all(&message.encode()).await?;

    println!("Sent ENABLE_RO_SPEC for ROSpec ID: {}", rospec_id);

    Ok(())
  }

  pub async fn send_start_rospec(&mut self, rospec_id: u32, receive_response: Option<bool>, expected_response_t: u16, response_callback: Option<Box<dyn Fn(LlrpResponse)>>) -> Result<(), Box<dyn Error>> {
    let message_id = self.next_message_id();

    let message = LlrpMessage::new_start_rospec(message_id, rospec_id);
    self.stream.write_all(&message.encode()).await?;

    println!("Sent START_RO_SPEC for ROSpec ID: {}", rospec_id);

    /*
    if receive_response.is_some_and(|x| x == true) {
      let response = self.receive_response().await?;
      if let Some(cb) = response_callback { cb(response) }
    }
    */

    Ok(())
  }

  pub async fn send_stop_rospec(&mut self, rospec_id: u32) -> Result<(), Box<dyn Error>> {
    let message_id = self.next_message_id();

    let message = LlrpMessage::new_stop_rospec(message_id, rospec_id);
    self.stream.write_all(&message.encode()).await?;

    println!("Sent STOP_RO_SPEC for ROSpec ID: {}", rospec_id);

    Ok(())
  }

  pub async fn send_delete_rospec(&mut self, rospec_id: u32) -> Result<(), Box<dyn Error>> {
    let message_id = self.next_message_id();

    let message = LlrpMessage::new_delete_rospec(message_id, rospec_id);
    self.stream.write_all(&message.encode()).await?;

    println!("Sent DELETE_RO_SPEC for ROSpec ID: {}", rospec_id);

    let response = self.receive_response().await?;
    self.check_expected_response_type(LlrpMessageType::DeleteROspecResponse, response.message_type);

    Ok(())
  }

  fn check_expected_response_type(&mut self, expected_response_type: LlrpMessageType, response_type: LlrpMessageType) {
    if expected_response_type != expected_response_type {
      println!("Warning: Expected response type {:?}, received {}", get_message_type_str(expected_response_type.value()), get_message_type_str(response_type.value()));
    } else {
      println!("Received response: {}", get_message_type_str(expected_response_type.value()));
    }
  }

  async fn receive_response(&mut self) -> Result<LlrpResponse, Box<dyn Error>> {
    let mut buf = BytesMut::with_capacity(1024);

    let result = timeout(Duration::from_millis(self.response_timeout), async {

      // Read header (10 bytes)
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

      // Read the entire message (header + payload)
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

    // Handle timeout
    if let Err(_) = result {
      return Err(Box::new(std::io::Error::new(
        std::io::ErrorKind::TimedOut,
        "Timeout while waiting for response",
      )));
    }

    // Decode the message
    let llrp_message = LlrpMessage::decode(&mut buf)?;
    let llrp_response = LlrpResponse::from_message(llrp_message);

    Ok(llrp_response)
  }

}