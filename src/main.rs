mod llrp;
mod client;
mod config;

use std::env;
use log::{info, debug, error};
use tokio::{self};

use client::LlrpClient;

#[tokio::main]
async fn main() {

  let current_dir = env::current_dir().unwrap();
  let config_file = current_dir.join("config.json");

  let get_reader_capabilities  = false;
  let get_reader_config        = false;

  match LlrpClient::initialize(config_file.to_str().unwrap()).await {
    Ok(mut client) => {

      if get_reader_capabilities {
        if let Err(e) = client.send_get_reader_capabilities().await {
          error!("GetReaderCapabilities error: {}", e)
        }
      }

      if let Err(e) = client.send_delete_rospec(0).await {
        error!("DeleteROSpec error: {}", e);
      }
      
      if let Err(e) = client.send_set_reader_config().await {
        error!("SetReaderConfig error: {}", e);
      }

      if get_reader_config {
        if let Err(e) = client.send_get_reader_config().await {
          error!("GetReaderConfig error: {}", e);
        }
      }

      if let Err(e) = client.send_enable_events_and_reports().await {
        error!("EnableEventsAndReports error: {}", e);
      }

      if let Err(e) = client.send_add_rospec().await {
        error!("AddROSpec error: {}", e);
      }

      if let Err(e) = client.send_enable_rospec().await {
        error!("EnableROSpec error: {}", e);
      }

      if let Err(e) = client.send_start_rospec().await {
        error!("StartROSpec error: {}", e);
      }

      if let Err(e) = client.await_ro_access_report( | response | async move {
        
        match response.decode() {
          
          Ok(tag_reports) => {
            for tag_report in tag_reports {
              debug!("[EPC] {}", tag_report);
            }
          }

          Err(e) => {
            error!("ROAccessReport decoding error: {}", e);
          }
        }

      }).await {
        error!("Error while attempting to receive ROAccessReport: {}", e)
      }

      if let Err(e) = client.send_stop_rospec().await {
        error!("StopROSpec error: {}", e);
      }
      
      if let Err(e) = client.send_close_connection().await {
        error!("CloseConnection error: {}", e);
      }
    }

    Err(e) => {
      error!("Failed to connect to LLRP server: {}", e)
    }
  }
}