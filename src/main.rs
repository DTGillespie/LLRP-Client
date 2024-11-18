mod llrp;
mod client;

use std::time::{Duration, Instant};

use client::LlrpClient;
use tokio::{self};

#[tokio::main]
async fn main() {
  let addr = "192.168.1.102:5084";

  match LlrpClient::connect(addr).await {
    Ok(mut client) => {
      println!("Connected to LLRP reader: {}", addr);

      let rospec_id = 1;

      if let Err(e) = client.send_delete_rospec(0x00).await {
        eprintln!("Failed to send DELETE_RO_SPEC: {}", e);
      }
      
      if let Err(e) = client.send_enable_events_and_reports().await {
        eprintln!("Failed to send ENABLE_EVENTS_AND_REPORTS: {}", e);
      }

      if let Err(e) = client.send_add_rospec(rospec_id).await {
        eprintln!("Failed to send ROSpec: {}", e);
      }

      if let Err(e) = client.send_enable_rospec(rospec_id).await {
        eprintln!("Failed to send ENABLE_RO_SPEC: {}", e);
      }

      if let Err(e) = client.send_start_rospec(rospec_id, None, None).await {
        eprintln!("Failed to send StartROSpec: {}", e)
      }

      /*
      if let Err(e) = client.send_start_rospec(
        rospec_id,
        Some(true),
        Some(Box::new(|res| {
          println!("Debug res_cb, received response: {:?}", res);
        }))
      ).await {
        eprintln!("Failed to send StartROSpec: {}", e)
      }
      */

      let mut last_keep_alive = Instant::now();
      let loop_start = Instant::now();
      loop {
        if loop_start.elapsed().as_millis() >= 1000 { break }

        /*
        if last_keep_alive.elapsed().as_millis() >= 100 {
          if let Err(e) = client.send_keep_alive().await {
            eprintln!("Failed to send KEEP_ALIVE: {}", e);
          }
          last_keep_alive = Instant::now();
        }
        */

        tokio::time::sleep(Duration::from_millis(25)).await;
      }

      if let Err(e) = client.send_stop_rospec(rospec_id).await {
        eprintln!("Failed to send StopROSpec: {}", e)
      }
      
      if let Err(e) = client.disconnect().await {
        eprintln!("Failed to send CLOSE_CONNECTION: {}", e);
      }
    }

    Err(e) => {
      eprintln!("Failed to connect to LLRP reader: {}", e)
    }
  }
}