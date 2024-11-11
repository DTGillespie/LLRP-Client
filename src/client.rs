use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use bytes::{BytesMut, Buf, BufMut};
use std::error::Error;
use crate::llrp::{
  LlrpMessage, TagReport, ADD_ROSPEC_RESPONSE, GET_SUPPORTED_VERSION_RESPONSE, RO_ACCESS_REPORT, SET_PROTOCOL_VERSION_RESPONSE
};

pub struct LlrpClient {
  stream: TcpStream,
}

impl LlrpClient {
  
  pub async fn connect(addr: &str) -> Result<Self, Box<dyn Error>> {
    let stream = TcpStream::connect(addr).await?;
    Ok(LlrpClient { stream })
  }

  async fn send_message(&mut self, message: &LlrpMessage) -> Result<(), Box<dyn Error>> {
    let message_buf = message.encode();
    self.stream.write_all(&message_buf).await?;
    Ok(())
  }

  pub async fn perform_version_negotiation(&mut self, message_id: &mut u32) -> Result<(), Box<dyn Error>> {
    let get_supported_version = LlrpMessage::new_get_supported_version(*message_id);
    *message_id += 1;
    let encoded_msg = get_supported_version.encode(1);
    self.stream.write_all(&encoded_msg).await?;
    println!("Sent GET_SUPPORTED_VERSION");

    let response = self.receive_message().await?;
    if response.message_type != GET_SUPPORTED_VERSION_RESPONSE {
      return Err(Box::new(io::Error::new(io::ErrorKind::InvalidData, "Expected GET_SUPPORTED_VERSION_RESPONSE")));
    }

    /* Can add additional logic for supported multiple versions, using version 1 by default */
    let negotiated_version = 1;

    let set_protocol_version = LlrpMessage::new_set_protocol_version(message_id, negotiated_version);
    *message_id+=1;
    let encoded_message = set_protocol_version.encode(1);
    self.stream.write_all(&encoded_message).await?;
    println!("Send SET_PROTOCOL_VERSION with version {}", negotiated_version);

    let response = self.receive_message().await?;
    if response.message_type != SET_PROTOCOL_VERSION_RESPONSE {
      return Err(Box::new(io::Error::neW(io::ErrorKind::InvalidData, "Expected SET_PROTOCOL_VERSION_RESPONSE")));
    }

    // Check LLRPStatus in the response
    // (Implement parsing logic to verify that the protocol version was accepte 
    println!("Version negotiation successful, using protocol version {}", negotiated_version);
    Ok(negotiated_version)
  }

  pub async fn send_enable_events_and_reports(&mut self, message_id: u32) -> Result<(), Box<dyn Error>> {
    let message = LlrpMessage::new_enable_events_and_reports(message_id);
    self.send_message(&message).await?;
    println!("Sent ENABLE_EVENTS_AND_REPORTS");
    Ok(())
  }

  pub async fn send_add_rospec(&mut self, message_id: u32, rospec_id: u32) -> Result<(), Box<dyn Error>> {
    let message = LlrpMessage::new_add_rospec(message_id, rospec_id);
    self.send_message(&message).await?;
    println!("Sent ADD_ROSPEC with ID: {}", rospec_id);
    
    /*
    let response = self.receive_message().await?;

    if response.message_type == ADD_ROSPEC_RESPONSE {
      println!("Received ADD_ROSPEC_RESPONSE");
    } else {
      println!("Unexpected message type: {}", response.message_type)
    }
    */

    Ok(())
  }

  pub async fn send_start_rospec(&mut self, message_id: u32, rospec_id: u32) -> Result<(), Box<dyn Error>> {
    let message = LlrpMessage::new_start_rospec(message_id, rospec_id);
    self.send_message(&message).await?;
    println!("Send START_ROSPEC for ROSpec ID: {}", rospec_id);
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

    if llrp_message.message_type == RO_ACCESS_REPORT {
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