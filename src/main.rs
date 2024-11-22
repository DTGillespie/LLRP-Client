mod llrp;
mod client;
mod config;

use std::env;

use config::load_config;
use tokio::{self};
use client::LlrpClient;

#[tokio::main]
async fn main() {

  let current_dir = env::current_dir().unwrap();
  let config_file = current_dir.join("llrp_config.json");

  let config = match load_config(config_file.to_str().unwrap()) {
    Ok(cfg) => cfg,
    Err(e) => {
      eprintln!("Failed to load LLRP configuration: {}", e);
      return;
    }
  };
  
  let host = &config.host;
  let res_timeout = config.res_timeout;
  let debug = config.debug;
  let get_reader_capabilities = config.get_reader_capabilities;

  match LlrpClient::connect(host, res_timeout, debug).await {
    Ok(mut client) => {

      let await_response_ack = Some(debug);
      
      if get_reader_capabilities {
        if let Err(e) = client.send_get_reader_capabilities().await {
          eprintln!("Error during GetReaderCapabilities operation: {}", e)
        }
      }

      if let Err(e) = client.send_delete_rospec(0x00, await_response_ack).await {
        eprintln!("Error during DeleteROSpec operation: {}", e);
      }
      
      if let Err(e) = client.send_set_reader_config(await_response_ack).await {
        eprintln!("Error during SendReaderConfig operation: {}", e);
      }

      if let Err(e) = client.send_enable_events_and_reports(await_response_ack).await {
        eprintln!("Error during EnableEventsAndReports operation: {}", e);
      }

      if let Err(e) = client.send_add_rospec(&config.ROSpec, await_response_ack).await {
        eprintln!("Error during AddROSpec operation: {}", e);
      }

      if let Err(e) = client.send_enable_rospec(config.ROSpec.rospec_id, await_response_ack).await {
        eprintln!("Error during EnableROSpec operation: {}", e);
      }

      if let Err(e) = client.send_start_rospec(config.ROSpec.rospec_id, await_response_ack).await {
        eprintln!("Error during StartROSpec operation: {}", e);
      }

      if let Err(e) = client.await_ro_access_report(5, | response | async move {
        
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

      if let Err(e) = client.send_stop_rospec(config.ROSpec.rospec_id, await_response_ack).await {
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