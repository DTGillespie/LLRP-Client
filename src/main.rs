mod llrp;
mod client;

use std::{ptr::null, time::{Duration, Instant}};

use client::LlrpClient;
use tokio;
use crate::llrp::{RO_ACCESS_REPORT};

#[tokio::main]
async fn main() {
  let addr = "192.168.1.102:5084";

  match LlrpClient::connect(addr).await {
    Ok(mut client) => {
      println!("Connected to LLRP reader: {}", addr);

      let mut protocol_version = -1;
      let rospec_id = 1;
      let mut message_id = 1001;
      let mut next_message_id = || {
        let current_id = message_id;
        message_id += 1;
        current_id
      };

      match client.perform_version_negotiation(&mut message_id).await {
        Ok(protocol_version) => {
          protocol_version = protocol_version;
        },
        Err(e) => {
          eprintln!("Version negotiation failed: {}", e);
          return;
        }
      }

      /*
      if let Err(e) = client.send_enable_events_and_reports(next_message_id()).await {
        eprintln!("Failed to send ENABLE_EVENTS_AND_REPORTS: {}", e);
      }

      if let Err(e) = client.send_add_rospec(next_message_id(), rospec_id).await {
        eprintln!("Failed to send ROSpec: {}", e);
      }

      if let Err(e) = client.send_start_rospec(next_message_id(), rospec_id).await {
        eprintln!("Failed to send StartROSpec: {}", e)
      }

      loop {
        
        /*
        if last_keep_alive.elapsed().as_secs() >= 5 {
          if let Err(e) = client.send_keep_alive(next_message_id()).await {
            eprintln!("Failed to send KEEP_ALIVE: {}", e);
          }
          last_keep_alive = Instant::now();
        }
        */

        match client.receive_message().await {
          Ok(msg) => {
            if msg.message_type == RO_ACCESS_REPORT {
              println!("Processed Tag Report.");
            }
          }
          Err(e) => {
            eprintln!("Error receiving message: {}", e);
            break;
          }
        }

        tokio::time::sleep(Duration::from_millis(25)).await;
      }
      */
    }

    Err(e) => {
      eprintln!("Failed to connect to LLRP reader: {}", e)
    }
  }
}