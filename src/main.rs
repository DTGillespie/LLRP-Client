mod llrp;
mod client;

use client::LlrpClient;
use tokio::{self};

#[tokio::main]
async fn main() {

  let addr = "192.168.1.102:5084";
  let debug = false;

  match LlrpClient::connect(addr, 2500).await {
    Ok(mut client) => {
      println!("Connected to LLRP reader: {}", addr);

      let rospec_id = 1;
      let mut await_response_ack = Some(false);
      
      if debug {
        await_response_ack = Some(true);
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

      if let Err(e) = client.send_add_rospec(rospec_id, await_response_ack).await {
        eprintln!("Error during AddROSpec operation: {}", e);
      }

      if let Err(e) = client.send_enable_rospec(rospec_id, await_response_ack).await {
        eprintln!("Error during EnableROSpec operation: {}", e);
      }

      if let Err(e) = client.send_start_rospec(rospec_id, await_response_ack).await {
        eprintln!("Error during StartROSpec operation: {}", e);
      }

      if let Err(e) = client.await_ro_access_report(5, | response | async move {
        
        match response.decode() {
          
          Ok(tag_reports) => {
            for tag_report in tag_reports {
              println!("TagReportData: {}", tag_report);
            }
          }

          Err(e) => {
            eprintln!("Error decoding ROAccessReport: {}", e);
          }
        }

      }).await {
        println!("Error attempting to receive ROAccessReport: {}", e)
      }

      if let Err(e) = client.send_stop_rospec(rospec_id, await_response_ack).await {
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