use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use bytes::{BytesMut, Buf, BufMut};
use std::error::Error;
use crate::llrp::{LlrpMessage, TagReport, TYPE_RO_ACCESS_REPORT};

pub struct LlrpClient {
  stream: TcpStream,
}

impl LlrpClient {
  
  pub async fn connect(addr: &str) -> Result<Self, Box<dyn Error>> {
    let stream = TcpStream::connect(addr).await?;
    Ok(LlrpClient { stream })
  }

  pub async fn send_keep_alive(&mut self, message_id: u32) -> Result<(), Box<dyn Error>> {
    let message = LlrpMessage::new(62, message_id, vec![]);
    self.stream.write_all(&message.encode()).await?;
    println!("Sent KEEP_ALIVE");
    Ok(())
  }

  pub async fn send_enable_events_and_reports(&mut self, message_id: u32) -> Result<(), Box<dyn Error>> {
    let message = LlrpMessage::new_enable_events_and_reports(message_id);
    self.stream.write_all(&message.encode()).await?;
    println!("Sent ENABLE_EVENTS_AND_REPORTS");
    Ok(())
  }

  pub async fn send_add_rospec(&mut self, message_id: u32, rospec_id: u32) -> Result<(), Box<dyn Error>> {
    let message = LlrpMessage::add_rospec(message_id, rospec_id);
    self.stream.write_all(&message.encode()).await?;
    println!("Sent ROSpec with ID: {}", rospec_id);
    Ok(())
  }

  pub async fn send_start_rospec(&mut self, message_id: u32, rospec_id: u32) -> Result<(), Box<dyn Error>> {
    let message = LlrpMessage::new_start_rospec(message_id, rospec_id);
    self.stream.write_all(&message.encode()).await?;
    println!("Sent StartROSpec for ROSpec ID: {}", rospec_id);
    Ok(())
  }

  pub async fn send_stop_rospec(&mut self, message_id: u32, rospec_id: u32) -> Result<(), Box<dyn Error>> {
    let message = LlrpMessage::new_stop_rospec(message_id, rospec_id);
    self.stream.write_all(&message.encode()).await?;
    println!("Sent StopROSpec for ROSpec ID: {}", rospec_id);
    Ok(())
  }

  pub async fn send_delete_rospec(&mut self, message_id: u32, rospec_id: u32) -> Result<(), Box<dyn Error>> {
    let message = LlrpMessage::new_delete_rospec(message_id, rospec_id);
    self.stream.write_all(&message.encode()).await?;
    println!("Sent DeleteROSpec for ROSpec ID: {}", rospec_id);
    Ok(())
  }

  pub async fn send_close_connection(&mut self, message_id: u32) -> Result<(), Box<dyn Error>> {
    let message = LlrpMessage::new_close_connection(message_id);
    self.stream.write_all(&message.encode()).await?;
    println!("Sent CLOSE_CONNECTION");
    Ok(())
  }

  pub async fn receive_message(&mut self) -> Result<LlrpMessage, Box<dyn Error>> {
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
    println!("Received message with type: {}", llrp_message.message_type);

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

    Ok(llrp_message)
  }
}