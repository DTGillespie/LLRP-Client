mod llrp;
mod client;

use std::time::{Duration, Instant};

use client::LlrpClient;
use tokio::{self};

#[tokio::main]
async fn main() {

  let addr = "192.168.1.102:5084";

  match LlrpClient::connect(addr, 2500).await {
    Ok(mut client) => {
      println!("Connected to LLRP reader: {}", addr);

      let rospec_id = 1;

      if let Err(e) = client.send_delete_rospec(0x00, Some(true)).await {
        eprintln!("Error during DeleteROSpec operation: {}", e);
      }
      
      if let Err(e) = client.send_set_reader_config(Some(true)).await {
        eprintln!("Error during SendReaderConfig operation: {}", e);
      }

      if let Err(e) = client.send_enable_events_and_reports(Some(true)).await {
        eprintln!("Error during EnableEventsAndReports operation: {}", e);
      }

      if let Err(e) = client.send_add_rospec(rospec_id, Some(true)).await {
        eprintln!("Error during AddROSpec operation: {}", e);
      }

      if let Err(e) = client.send_enable_rospec(rospec_id, Some(true)).await {
        eprintln!("Error during EnableROSpec operation: {}", e);
      }

      if let Err(e) = client.send_start_rospec(rospec_id, Some(true)).await {
        eprintln!("Error during StartROSpec operation: {}", e);
      }

      if let Err(e) = client.await_ro_access_report(5, |response| async move {
        println!("Received ROAccessReport: {:?}", response);
      }).await {
        println!("Error attempting to receive ROAccessReport: {}", e)
      }

      if let Err(e) = client.send_stop_rospec(rospec_id, Some(true)).await {
        eprintln!("Error during StopROSpec operation: {}", e);
      }
      
      if let Err(e) = client.disconnect(None).await {
        eprintln!("Error during CloseConnection operation: {}", e);
      }
    }

    Err(e) => {
      eprintln!("Failed to connect to LLRP server: {}", e)
    }
  }
}