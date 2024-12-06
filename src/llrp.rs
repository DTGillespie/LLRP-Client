use std::{collections::HashMap, fmt, io::{self, Error, ErrorKind}};
use strum_macros::{EnumIter, EnumString};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use strum::IntoEnumIterator;
use once_cell::sync::Lazy;
use log::{info, debug, warn, error};

use crate::{config::{ROSpecConfig, ReaderConfig}, params::{parse_parameters, C1G2LLRPCapabilities, GeneralDeviceCapabilities, Identification, LLRPCapabilities, LLRPStatus, LlrpParameterData, RegulatoryCapabilities, TagReportData}};

#[derive(Debug, EnumIter, EnumString, PartialEq, Eq, Hash, Copy, Clone)]
pub enum LlrpMessageType {
  None                          = 0,
  GetReaderCapabilities         = 1,
  GetReaderCapabilitiesResponse = 11,
  GetReaderConfig               = 2,
  GetReaderConfigResponse       = 12,
  SetReaderConfig               = 3,
  SetReaderConfigResponse       = 13,
  CloseConnection               = 14,
  CloseConnectionResponse       = 4,
  AddROSpec                     = 20,
  AddROspecResponse             = 30,
  DeleteROSpec                  = 21,
  DeleteROSpecResponse          = 31,
  StartROSpec                   = 22,
  StartROSpecResponse           = 32,
  StopROSpec                    = 23,
  StopROSpecResponse            = 33,
  EnableROSpec                  = 24,
  EnableROSpecResponse          = 34,
  DisableROSpec                 = 25,
  DisableROSpecResponse         = 35,
  GetROSpecs                    = 26,
  GetROSpecsResponse            = 36,
  GetReport                     = 60,
  ROAccessReport                = 61,
  Keepalive                     = 62,
  KeepaliveAck                  = 72,
  ReaderEventNotification       = 63,
  EnableEventsAndReports        = 64,
  ErrorMessage                  = 100,
}

impl LlrpMessageType {
  
  pub fn value(
    &self
  ) -> u16 {
    *self as u16
  }

  pub fn from_value(
    value: u16
  ) -> Option<Self> {
    Self::iter().find(|&variant| variant as u16 == value)
  }
}

static LLRP_MESSAGE_TYPE_LUT: 
Lazy<HashMap<u16, String>> = Lazy::new(|| {
  LlrpMessageType::iter()
    .map(|variant| (variant as u16, format!("{:?}", variant)))
    .collect()
});

pub fn get_message_type_str(
  message_type: u16
) -> &'static str {
  LLRP_MESSAGE_TYPE_LUT
    .get(&message_type)
    .map(|s| s.as_str())
    .unwrap_or("Unknown message type")
}

#[derive(Debug, EnumIter, EnumString, PartialEq, Eq, Hash, Copy, Clone)]
pub enum LlrpParameterType {
  UTCTimeStamp                      = 128,
  Uptime                            = 129,
  GeneralDeviceCapabilities         = 137,
  MaximumReceiveSensitivity         = 363,
  ReceiveSensitivityTableEntry      = 139,
  PerAntennaAirProtocol             = 140,
  GPIPCapabilities                  = 141,
  LLRPCapabilities                  = 142,
  RegulatoryCapabilities            = 143,
  UHFBandCapabilities               = 144,
  TransmitPowerLevelTableEntry      = 145,
  FrequencyInformation              = 146,
  FrequencyHopTable                 = 147,
  FixedFrequencyTable               = 148,
  PerAntennaReceiveSensitivityRange = 149,
  RFSurveyFrequencyCapabilities     = 365,
  ROSpec                            = 177,
  ROBoundarySpec                    = 178,
  ROSpecStartTrigger                = 179,
  PeriodicTriggerValue              = 180,
  GPITriggerValue                   = 181,
  ROSpecStopTrigger                 = 182,
  AISpec                            = 183,
  AISpecStopTrigger                 = 184,
  TagObservationTrigger             = 185,
  InventoryParameterSpec            = 186,
  RFSurveySpec                      = 187,
  RFSurveySpecStopTrigger           = 188,
  LoopSpec                          = 355,
  AccessSpec                        = 207,
  AccessSpecStopTrigger             = 208,
  AccessCommand                     = 209,
  ClientRequestOpSpec               = 210,
  ClientRequestResponse             = 211,
  LLRPConfigurationStateValue       = 217,
  Identification                    = 218,
  GPOWriteData                      = 219,
  KeepAliveSpec                     = 220,
  AntennaProperties                 = 221,
  AntennaConfiguration              = 222,
  RFReceiver                        = 223,
  RFTransmitter                     = 224,
  GPIPortCurrentState               = 225,
  EventsAndReports                  = 226,
  ROReportSpec                      = 237,
  TagReportContentSelector          = 238,
  TagReportData                     = 240,
  EPCData                           = 241,
  EPC96                             = 13,
  ReaderEventNotificationData       = 246,
  ConnAttemptEvent                  = 256,
  LLRPStatus                        = 287,
  C1G2LLRPCapabilities              = 327,
  C1G2UHFRFModeTable                = 328,
  C1G2UHFRFModeTableEntry           = 329,
  Custom                            = 1023,
}

impl LlrpParameterType {

  pub fn value(
    &self
  ) -> u16 {
    *self as u16
  }

  pub fn from_value(
    value: u16
  ) -> Option<Self> {
    Self::iter().find(|&variant| variant as u16 == value)
  } 
}

/// Represents an LLRP-compliant message.
///
/// This struct encapsulates the core components of an LLRP message,
/// including its type, length, ID, and payload.
///
/// Fields:
/// - `message_type`: The type of the LLRP message.
/// - `message_length`: The total length of the message, including the header and payload.
/// - `message_id`: A unique identifier for the message.
/// - `payload`: The binary payload of the message.
#[derive(Debug)]
pub struct LlrpMessage {
  pub message_type   : LlrpMessageType,
  pub message_length : u32,
  pub message_id     : u32,
  pub payload        : Vec<u8>
}

/// Represents a basic LLRP TLV (Type-Length-Value) parameter.
///
/// This structure supports nested parameters, allowing complex
/// parameter hierarchies to be constructed and encoded.
///
/// Fields:
/// - `param_type`: LlrpParameterType enumerator.
/// - `payload`: A vector of nested `Parameter` instances.
#[derive(Debug)]
struct Parameter {
  param_type : LlrpParameterType, 
  payload    : Vec<Parameter>,
}

impl LlrpMessage {
  
  /// Constructs a new LLRP message with the specified type, ID, and payload.
  ///
  /// Automatically calculates the message length based on the payload size.
  pub fn new(
    message_type : LlrpMessageType, 
    message_id   : u32, 
    payload      : Vec<u8>
  ) -> Self {

    let message_length = 10 + payload.len() as u32;

    LlrpMessage {
      message_type,
      message_length,
      message_id,
      payload
    }
  }

  /// Constructs a new `EnableEventsAndReports` message.
  ///
  /// This message enables event and report generation on the reader.
  pub fn new_enable_events_and_reports(
    message_id: u32
  ) -> Self {
    LlrpMessage::new(LlrpMessageType::EnableEventsAndReports, message_id, vec![])
  }

  pub fn new_get_reader_capabilities(
    message_id: u32
  ) -> Self {

    let mut payload = BytesMut::new();
    payload.put_u8(0);

    LlrpMessage::new(LlrpMessageType::GetReaderCapabilities, message_id, payload.to_vec())
  }

  pub fn new_get_reader_config(
    message_id : u32,
  ) -> Self {

    let mut payload = BytesMut::new();

    payload.put_u16(0);
    payload.put_u8(0);
    payload.put_u16(0);
    payload.put_u16(0);

    LlrpMessage::new(LlrpMessageType::GetReaderConfig, message_id, payload.to_vec())
  }
  
  /// Constructs a new `SetReaderConfig` message
  /// 
  /// This message resets reader configuration to factory settings.
  pub fn new_set_reader_config(
    message_id : u32,
    config     : &ReaderConfig,
  ) -> Self {

    let rf_receiver = Parameter {
      param_type: LlrpParameterType::RFReceiver,
      payload: vec![]
    };

    let rf_transmitter = Parameter {
      param_type: LlrpParameterType::RFTransmitter,
      payload: vec![]
    };
    
    let antenna_configuration = Parameter {
      param_type: LlrpParameterType::AntennaConfiguration,
      payload: vec![rf_receiver, rf_transmitter]
    };

    let mut payload = BytesMut::new();

    payload.put_u8(128); // ResetToFactoryDefault (First bit is boolean value)

    fn encode_parameter(
      param     : &Parameter, 
      buffer    : &mut BytesMut,
      config    : &ReaderConfig
    ) {
      
      let initial_length_pos = buffer.len();

      buffer.put_u16(param.param_type.value());
      buffer.put_u16(0); // Length (dynamic)

      match param.param_type {

        LlrpParameterType::AntennaConfiguration => {
          buffer.put_u16(0); // Antenna ID (0 - All)
        } 

        LlrpParameterType::RFReceiver => {
          buffer.put_u16(config.rx_power_table_index); // Receive Sensitivity Table-index
        }

        LlrpParameterType::RFTransmitter => {
          buffer.put_u16(config.hop_table_id);         // HopTableId
          buffer.put_u16(config.channel_index);        // ChannelIndex
          buffer.put_u16(config.tx_power_table_index); // Transmit Power Table-index
        }

        _ => {}
      }

      for sub_param in &param.payload {
        encode_parameter(sub_param, buffer, config); 
      }

      let final_length_pos = buffer.len();
      let actual_length = (final_length_pos - initial_length_pos) as u16;

      buffer[initial_length_pos + 2..initial_length_pos + 4].copy_from_slice(&actual_length.to_be_bytes());
    };

    encode_parameter(&antenna_configuration, &mut payload, config);

    LlrpMessage::new(LlrpMessageType::SetReaderConfig, message_id, payload.to_vec())
  }

  /// Constructs a new `AddROSpec` message with the specified ROSpec ID.
  ///
  /// The ROSpec includes the following parameters:
  /// - `ROBoundarySpec`: Specifies start and stop triggers.
  /// - `AISpec`: Defines antenna configurations and stop triggers.
  /// - `ROReportSpec`: Configures report generation.
  pub fn new_add_rospec(
    message_id : u32, 
    config     : &ROSpecConfig
  ) -> Self {
    
    let ro_boundary_spec = Parameter {
      param_type: LlrpParameterType::ROBoundarySpec,
      payload: vec![]
    };

    let ai_spec = Parameter {
      param_type: LlrpParameterType::AISpec,
      payload: vec![]
    };

    let ro_report_spec = Parameter {
      param_type: LlrpParameterType::ROReportSpec,
      payload: vec![]
    };

    let ro_spec = Parameter {
      param_type: LlrpParameterType::ROSpec,
      payload: vec![ro_boundary_spec, ai_spec, ro_report_spec]
    };

    let mut payload = BytesMut::new();

    fn encode_parameter(
      param     : &Parameter, 
      buffer    : &mut BytesMut,
      config    : &ROSpecConfig
    ) {
      
      let initial_length_pos = buffer.len();
      buffer.put_u16(param.param_type.value());
      buffer.put_u16(0); // Length (dynamic)

      match param.param_type {
 
        LlrpParameterType::ROSpec => {
          buffer.put_u32(config.rospec_id);
          buffer.put_u8(config.priority); // Priority
          buffer.put_u8(0);               // CurrentState
        }

        LlrpParameterType::ROBoundarySpec => {

          // ROSpecStartTrigger
          buffer.put_u16(LlrpParameterType::ROSpecStartTrigger.value());
          buffer.put_u16(5); // Length (static)

          /* Fields */
          buffer.put_u8(config.ROSpecStartTriggerType); // ROSpecStartTriggerType

          // ROSpecStopTrigger
          buffer.put_u16(LlrpParameterType::ROSpecStopTrigger.value());
          buffer.put_u16(9); // Length (static)
          
          /* Fields */
          buffer.put_u8(config.ROSpecStopTriggerType);  // ROSpecStopTriggerType (0 - No stop trigger)
          buffer.put_u32(0); // Null-field padding (Fields not required with ROSpecStoTriggerType=0)
        }

        LlrpParameterType::AISpec => {

          // Antenna configuration
          buffer.put_u16(config.antenna_count);

          // AntennaID Array (Allocated before AISpecStopTrigger)
          for antenna_id in &config.antennas {
            buffer.put_u16(*antenna_id);
          }

          // AISpecStopTrigger
          buffer.put_u16(LlrpParameterType::AISpecStopTrigger.value());
          buffer.put_u16(9); // Length (dynamic)

          /* Fields */
          buffer.put_u8(config.AISpecStopTriggerType); // AISpecStopTriggerType
          buffer.put_u32(0); // Null-field padding

          // InventoryParamSpec
          buffer.put_u16(LlrpParameterType::InventoryParameterSpec.value());
          buffer.put_u16(7); // Length (static)

          buffer.put_u16(config.InventoryParamSpecID); // InventoryParamSpec ID
          buffer.put_u8(config.AIProtocol); // AiProcotol
        }

        LlrpParameterType::ROReportSpec => {

          buffer.put_u8(config.ROReportTriggerType); // ROReportTriggerType
          buffer.put_u16(config.ROReportTrigger_N);  // N

          // TagReportContentSelector
          buffer.put_u16(LlrpParameterType::TagReportContentSelector.value());
          buffer.put_u16(6); // Length (static)

          /* Fields */
          buffer.put_u16(config.ReportContentSelector); // ReportContentSelector (TagInfo/EPC)
        }

        _ => {}
      }

      // Recursively encode nested parameters.
      for sub_param in &param.payload {
        encode_parameter(sub_param, buffer, config); 
      }

      let final_length_pos = buffer.len();
      let actual_length = (final_length_pos - initial_length_pos) as u16;

      buffer[initial_length_pos + 2..initial_length_pos + 4].copy_from_slice(&actual_length.to_be_bytes());
    };

    encode_parameter(&ro_spec, &mut payload, config);

    LlrpMessage::new(LlrpMessageType::AddROSpec, message_id, payload.to_vec())
  }

  pub fn new_enable_rospec(
    message_id : u32, 
    rospec_id  : u32
  ) -> Self {

    let mut payload = BytesMut::with_capacity(4);
    payload.put_u32(rospec_id);
    
    LlrpMessage::new(LlrpMessageType::EnableROSpec, message_id, payload.to_vec())
  }

  pub fn new_start_rospec(
    message_id : u32, 
    rospec_id  : u32
  ) -> Self {

    let mut payload = BytesMut::with_capacity(4);
    payload.put_u32(rospec_id);
    
    LlrpMessage::new(LlrpMessageType::StartROSpec, message_id, payload.to_vec())
  }

  pub fn new_stop_rospec(
    message_id : u32, 
    rospec_id  : u32
  ) -> Self {
    
    let mut payload = BytesMut::with_capacity(4);
    payload.put_u32(rospec_id);
    
    LlrpMessage::new(LlrpMessageType::StopROSpec, message_id,   payload.to_vec())
  }

  pub fn new_delete_rospec(
    message_id : u32, 
    rospec_id  : u32
  ) -> Self {
    
    let mut payload = BytesMut::with_capacity(4);
    payload.put_u32(rospec_id);
    
    LlrpMessage::new(LlrpMessageType::DeleteROSpec, message_id, payload.to_vec())
  }

  /// Encodes the LLRP message into a binary format.
  ///
  /// This includes the LLRP header and the message payload.
  pub fn encode(
    &self
  ) -> BytesMut {

    let mut buffer = BytesMut::with_capacity(self.message_length as usize);

    let padding = 0;
    let version = 1;

    let version_and_type = ((padding & 0x7) << 13) | ((version & 0x7) << 10) | ((self.message_type.value()) & 0x3FFF);

    buffer.put_u16(version_and_type as u16);
    buffer.put_u32(self.message_length);
    buffer.put_u32(self.message_id);
    buffer.extend_from_slice(&self.payload);

    buffer
  }

  /// Decodes an LLRP message from a binary buffer.
  ///
  /// Returns an `io::Result` with the decoded message or an error.
  pub fn decode(
    buf: &mut BytesMut
  ) -> io::Result<Self> {

    if buf.len() < 10 {
      return Err(Error::new(ErrorKind::InvalidData, "Buffer too short for LLRP header"));
    }

    let version_and_type = buf.get_u16();
    let version = (version_and_type >> 10) & 0x7;
    let message_type_value = version_and_type & 0x3FF;
    let message_length = buf.get_u32();
    let message_id = buf.get_u32();

    if buf.len() < (message_length - 10) as usize {
      return Err(Error::new(ErrorKind::InvalidData, "Buffer too short for payload"));
    }

    let payload = buf.split_to((message_length - 10) as usize).to_vec();

    let message_type = LlrpMessageType::from_value(message_type_value)
      .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Unknown LLRP message type"))?;
    
    Ok(LlrpMessage {
      message_type,
      message_length,
      message_id,
      payload,
    })
  }
}

#[derive(Debug, Clone)]
pub struct LlrpResponse {
  pub message_type : LlrpMessageType,
  pub message_id   : u32,
  pub payload      : Vec<u8>
}

#[derive(Debug)]
pub enum LlrpResponseData {
  TagReport(Vec<TagReportData>),
  ReaderCapabilities(Vec<LlrpParameterData>),
  ReaderConfig(Vec<LlrpParameterData>),
}

impl LlrpResponse {
  
  pub fn from_message(
    message: LlrpMessage
  ) -> Self {
    LlrpResponse {
      message_type : message.message_type,
      message_id   : message.message_id,
      payload      : message.payload,
    }
  }

  pub fn decode(
    &self
  ) -> io::Result<LlrpResponseData> {
    let mut buf = BytesMut::from(&self.payload[..]);

    match self.message_type {

      LlrpMessageType::GetReaderCapabilitiesResponse => {

        let parameters = parse_parameters(&mut buf)?;
        let mut parsed_params: Vec<LlrpParameterData> = Vec::new();

        for param in parameters {
          match param.param_type {

            LlrpParameterType::LLRPStatus => {
              let llrp_status = LLRPStatus::decode(&param.param_value)?;
              info!("GetReaderCapabilitiesResponse->LLRPStatus: {:?}", llrp_status);
              parsed_params.push(LlrpParameterData::LLRPStatus(llrp_status));
            }

            LlrpParameterType::GeneralDeviceCapabilities => {
              let gdc = GeneralDeviceCapabilities::decode(&param.param_value)?;
              info!("GetReaderCapabilitiesResponse->GeneralDeviceCapabilities: {:?}", gdc);
              parsed_params.push(LlrpParameterData::GeneralDeviceCapabilities(gdc));
            }

            LlrpParameterType::LLRPCapabilities => {
              let llrp_caps = LLRPCapabilities::decode(&param.param_value)?;
              info!("GetReaderCapabilitiesResponse->LLRPCapabilities: {:?}", llrp_caps);
              parsed_params.push(LlrpParameterData::LLRPCapabilities(llrp_caps));
            }

            LlrpParameterType::RegulatoryCapabilities => {
              let reg_caps = RegulatoryCapabilities::decode(&param.param_value)?;
              info!("GetReaderCapabilitiesResponse->RegulatoryCapabilities: {:?}", reg_caps);
              parsed_params.push(LlrpParameterData::RegulatoryCapabilities(reg_caps));
            }

            LlrpParameterType::C1G2LLRPCapabilities=> {
              let c1g2_llrp_caps = C1G2LLRPCapabilities::decode(&param.param_value)?;
              info!("GetReaderCapabilitiesResponse->C1G2LLRPCapabilities: {:?}", c1g2_llrp_caps);
              parsed_params.push(LlrpParameterData::C1G2LLRPCapabilities(c1g2_llrp_caps));
            }

            _ => {
              warn!("Unhandled GetReaderCapabilitiesResponse parameter: {:?}", param.param_type);
            }
          }
        }

        Ok(LlrpResponseData::ReaderCapabilities(parsed_params))
      }

      LlrpMessageType::GetReaderConfigResponse => {

        let parameters = parse_parameters(&mut buf)?;
        let mut parsed_params: Vec<LlrpParameterData> = Vec::new();

        for param in parameters {
          match param.param_type {

            LlrpParameterType::LLRPStatus => {
              let llrp_status = LLRPStatus::decode(&param.param_value)?;
              info!("GetReaderConfigResponse->LLRPStatus: {:?}", llrp_status);
              parsed_params.push(LlrpParameterData::LLRPStatus(llrp_status));
            }

            LlrpParameterType::Identification => {
              let identification = Identification::decode(&param.param_value)?;
              info!("GetReaderConfigResponse->Identification: {:?}", identification);
              parsed_params.push(LlrpParameterData::Identification(identification))
            }

            _ => {
              warn!("Unhandled GetReaderConfigResponse parameter: {:?}", param.param_type);
            }
          }
        }

        Ok(LlrpResponseData::ReaderConfig(parsed_params))
      }

      LlrpMessageType::ROAccessReport => {

        let mut tag_reports = Vec::new();
        let parameters = parse_parameters(&mut buf)?;

        for parameter in parameters {
          match parameter.param_type {

            LlrpParameterType::TagReportData => {
              let tag_report_data = TagReportData::decode(&parameter.param_value)?;
              tag_reports.push(tag_report_data);
            }

            _ => {
              warn!("Unhandled parameter type in ROAccessReport: {:?}", parameter.param_type);
            }
          }
        }

        Ok(LlrpResponseData::TagReport(tag_reports))
      }

      _ => {
        Err(io::Error::new(
          io::ErrorKind::InvalidData,
          format!("Unsupported message type: {:?}", self.message_type)
        ))
      }
    }
  }
}

#[derive(Debug)]
pub struct LlrpParameter {
  pub param_type   : LlrpParameterType,
  pub param_length : u16,
  pub param_value  : Vec<u8>,
  pub sub_params   : Option<Vec<LlrpParameter>>
}