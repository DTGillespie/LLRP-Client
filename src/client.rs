use bytes::BytesMut;
use tokio::io::{self, AsyncWriteExt, AsyncReadExt};
use tokio::net::TcpStream;
use std::error::Error;
use bytes::Buf;

use crate::llrp::LlrpMessage;
use crate::llrp::{
  TYPE_KEEPALIVE, 
  TYPE_CLOSE_CONNECTION,
  TYPE_ADD_ROSPEC_RESPONSE,
};

pub struct LlrpClient {
  stream: TcpStream,
  message_id: u32
}

impl LlrpClient {

  pub async fn connect(addr: &str) -> io::Result<Self> {
    let stream = TcpStream::connect(addr).await?;
    println!("Client connected: {}", addr);
    
    Ok(LlrpClient { stream, message_id: 1001 })
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

    let message = LlrpMessage::new(TYPE_CLOSE_CONNECTION, message_id, vec![]);
    self.stream.write_all(&message.encode()).await?;

    println!("Sent CLOSE_CONNECTION");

    //self.receive_response().await?;

    Ok(())
  }

  pub async fn send_keep_alive(&mut self) -> Result<(), Box<dyn Error>> {
    let message_id = self.next_message_id();

    let message = LlrpMessage::new(TYPE_KEEPALIVE, message_id, vec![]);
    self.stream.write_all(&message.encode()).await?;
    
    println!("Sent KEEP_ALIVE");

    //self.receive_response().await?;
    
    Ok(())
  }

  pub async fn send_enable_events_and_reports(&mut self) -> Result<(), Box<dyn Error>> {
    let message_id = self.next_message_id();
    
    let message = LlrpMessage::new_enable_events_and_reports(message_id);
    self.stream.write_all(&message.encode()).await?;
    
    println!("Sent ENABLE_EVENTS_AND_REPORTS");
    
    //self.receive_response().await?;

    Ok(())
  }

  pub async fn send_add_rospec(&mut self, rospec_id: u32) -> Result<(), Box<dyn Error>> {
    let message_id = self.next_message_id();
    
    let message = LlrpMessage::new_add_rospec(message_id, rospec_id);
    self.stream.write_all(&message.encode()).await?;
    
    println!("Sent ADD_ROSPEC with ID: {}", rospec_id);

    //self.receive_response().await?;

    Ok(())
  }

  pub async fn send_start_rospec(&mut self, rospec_id: u32) -> Result<(), Box<dyn Error>> {
    let message_id = self.next_message_id();

    let message = LlrpMessage::new_start_rospec(message_id, rospec_id);
    self.stream.write_all(&message.encode()).await?;

    println!("Sent START_RO_SPEC for ROSpec ID: {}", rospec_id);

    //self.receive_response().await?;

    Ok(())
  }

  pub async fn send_stop_rospec(&mut self, rospec_id: u32) -> Result<(), Box<dyn Error>> {
    let message_id = self.next_message_id();

    let message = LlrpMessage::new_stop_rospec(message_id, rospec_id);
    self.stream.write_all(&message.encode()).await?;

    println!("Sent STOP_RO_SPEC for ROSpec ID: {}", rospec_id);

    //self.receive_response().await?;

    Ok(())
  }

  pub async fn send_delete_rospec(&mut self, rospec_id: u32) -> Result<(), Box<dyn Error>> {
    let message_id = self.next_message_id();

    let message = LlrpMessage::new_delete_rospec(message_id, rospec_id);
    self.stream.write_all(&message.encode()).await?;

    println!("Sent DELETE_RO_SPEC for ROSpec ID: {}", rospec_id);

    Ok(())
  }
  
  /*
  async fn receive_response(&mut self) -> Result<(), Box<dyn Error>> {
    let mut buffer = vec![0u8; 1024];
    let n = self.stream.read(&mut buffer).await?;

    println!("Received response: {:?}", &buffer[..n]);

    Ok(())
  }
  */

  pub async fn receive_response(&mut self) -> Result<LlrpMessage, Box<dyn Error>> {
    let mut buf = BytesMut::with_capacity(1024);

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

    // Read buffer length without consuming bytes
    let message_length = {
      let mut header_buf = buf.clone();
      header_buf.get_u16();  // Skip message type
      header_buf.get_u16();  // Skip reserved
      header_buf.get_u32()   // Extract message length
    };

    // Read entire message (header + payload)
    while buf.len() < message_length as usize {
      let n = self.stream.read_buf(&mut buf).await?;
      if n == 0 {
        return Err(Box::new(std::io::Error::new(
          std::io::ErrorKind::UnexpectedEof,
          "Connection closed",
        )));
      }
    }

    let llrp_message = LlrpMessage::decode(&mut buf)?;
    println!("Received message of type: {}", llrp_message.message_type);

    /*
    if llrp_message.message_type == TYPE_RO_ACCESS_REPORT {
      println!("Received Tag Report:");
      let mut payload_buf = BytesMut::from(&llrp_message.payload[..]);
      while !payload_buf.is_empty() {
        if let Ok(tag_report) = TagReport::decode(&mut payload_buf) {
          println!("Tag EPC: {:?}", tag_report.epc);
          println!("Timestamp: {}", tag_report.timestamp);
        } else {
          println!("Error decoding Tag Report.");
        }
      }
    } else {
      println!("Non-tag report message received, type: {}", llrp_message.message_type);
    }
    */

    Ok(llrp_message)
  }
}