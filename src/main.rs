mod llrp;
mod client;
mod config;

use std::env;

use tokio::{self};
use client::LlrpClient;

#[tokio::main]
async fn main() {

  let current_dir = env::current_dir().unwrap();
  let config_file = current_dir.join("llrp_config.json");

  let get_reader_capabilities  = false;
  let get_reader_config        = false;

  match LlrpClient::initialize(config_file.to_str().unwrap()).await {
    Ok(mut client) => {

      if get_reader_capabilities {
        if let Err(e) = client.send_get_reader_capabilities().await {
          eprintln!("Error during GetReaderCapabilities operation: {}", e)
        }
      }

      if let Err(e) = client.send_delete_rospec(0).await {
        eprintln!("Error during DeleteROSpec operation: {}", e);
      }
      
      if let Err(e) = client.send_set_reader_config().await {
        eprintln!("Error during SetReaderConfig operation: {}", e);
      }

      if get_reader_config {
        if let Err(e) = client.send_get_reader_config().await {
          eprintln!("Error during GetReaderConfig operation: {}", e);
        }
      }

      if let Err(e) = client.send_enable_events_and_reports().await {
        eprintln!("Error during EnableEventsAndReports operation: {}", e);
      }

      if let Err(e) = client.send_add_rospec().await {
        eprintln!("Error during AddROSpec operation: {}", e);
      }

      if let Err(e) = client.send_enable_rospec().await {
        eprintln!("Error during EnableROSpec operation: {}", e);
      }

      if let Err(e) = client.send_start_rospec().await {
        eprintln!("Error during StartROSpec operation: {}", e);
      }

      if let Err(e) = client.await_ro_access_report( | response | async move {
        
        match response.decode() {
          
          Ok(tag_reports) => {
            for tag_report in tag_reports {
              println!("[EPC] {}", tag_report);
            }
          }

          Err(e) => {
            eprintln!("Error decoding ROAccessReport: {}", e);
          }
        }

      }).await {
        println!("Error attempting to receive ROAccessReport: {}", e)
      }

      if let Err(e) = client.send_stop_rospec().await {
        eprintln!("Error during StopROSpec operation: {}", e);
      }
      
      if let Err(e) = client.send_close_connection().await {
        eprintln!("Error during CloseConnection operation: {}", e);
      }
    }

    Err(e) => {
      eprintln!("Failed to connect to LLRP server: {}", e)
    }
  }
}