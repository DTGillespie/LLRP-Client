use serde::{Deserialize, Serialize};
use std::fs;
use serde_json;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
  pub host                     : String,
  pub log_response_ack         : bool,
  pub response_timeout         : u64,
  pub ROSpec                   : ROSpecConfig
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ROSpecConfig {
  pub rospec_id              : u32,
  pub priority               : u8,
  pub antenna_count          : u16,
  pub antennas               : Vec<u16>,
  pub ROSpecStartTriggerType : u8,
  pub ROSpecStopTriggerType  : u8,
  pub AISpecStopTriggerType  : u8,
  pub InventoryParamSpecID   : u16,
  pub AIProtocol             : u8,
  pub ROReportTriggerType    : u8,
  pub ROReportTrigger_N      : u16,
  pub ReportContentSelector  : u16,
}

pub fn load_config(file_path: &str) -> Result<Config, Box<dyn std::error::Error>> {
  
  let config_data = fs::read_to_string(file_path)?;
  let config: Config = serde_json::from_str(&config_data)?;
  
  Ok(config)
}