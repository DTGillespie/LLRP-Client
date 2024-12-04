use std::{collections::HashMap, fmt, io::{self, Error, ErrorKind}};
use strum_macros::{EnumIter, EnumString};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use strum::IntoEnumIterator;
use once_cell::sync::Lazy;
use log::{info, debug, warn, error};

use crate::config::{ROSpecConfig, ReaderConfig};

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
  CustomMessage                 = 1023
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

static LLRP_PARAMETER_TYPE_LUT: 
Lazy<HashMap<u16, String>> = Lazy::new(|| {
  LlrpParameterType::iter()
    .map(|variant| (variant as u16, format!("{:?}", variant)))
    .collect()
});

pub fn get_parameter_type_str(
  message_type: u16
) -> &'static str {
  LLRP_MESSAGE_TYPE_LUT.get(&message_type)
  .map(|s| s.as_str())
  .unwrap_or(&"Unknown parameter type")
}

#[derive(Debug)]
pub enum LlrpParameterData {
  GeneralDeviceCapabilities(GeneralDeviceCapabilities),
  LLRPCapabilities(LLRPCapabilities),
  RegulatoryCapabilities(RegulatoryCapabilities)
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
              // Unhandled
              warn!("LLRPStatus parameter is currently unhandled");
            }

            LlrpParameterType::GeneralDeviceCapabilities => {
              let gdc = GeneralDeviceCapabilities::decode(&param.param_value)?;
              info!("GeneralDeviceCapabilities: {:?}", gdc);
              parsed_params.push(LlrpParameterData::GeneralDeviceCapabilities(gdc));
            }

            LlrpParameterType::LLRPCapabilities => {
              let llrp_caps = LLRPCapabilities::decode(&param.param_value)?;
              info!("LLRPCapabilities: {:?}", llrp_caps);
              parsed_params.push(LlrpParameterData::LLRPCapabilities(llrp_caps));
            }

            LlrpParameterType::RegulatoryCapabilities => {
              let reg_caps = RegulatoryCapabilities::decode(&param.param_value)?;
              info!("RegulatoryCapabilities: {:?}", reg_caps);
              parsed_params.push(LlrpParameterData::RegulatoryCapabilities(reg_caps));
            }

            LlrpParameterType::C1G2LLRPCapabilities=> {
              // Unhandled
              warn!("C1G2LLRPCapabilities parameter is currently unhandled");
            }

            _ => {
              warn!("Unhandled parameter: {:?}", param.param_type);
            }
          }
        }

        Ok(LlrpResponseData::ReaderCapabilities(parsed_params))
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

      /*
      LlrpMessageType::GetReaderConfigResponse => {
        let parameters = parse_parameters(&mut buf)?;
        Ok(LlrpResponseData::ReaderConfig(parameters))
      }
      */

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

pub fn parse_parameters(
  buf: &mut BytesMut
) -> io::Result<Vec<LlrpParameter>> {

  let mut parameters = Vec::new();

  while buf.remaining() > 0 {

    // Check if TLV or TV encoded
    let first_byte = buf[0];
    debug!("First byte: 0x{:02X}", first_byte);

    if (first_byte & 0x80) != 0 {

      debug!("Parsing TV parameter");

      let param_type_value = buf.get_u8();
      let param_type_value = (param_type_value & 0x7F) as u16;

      let param_type = LlrpParameterType::from_value(param_type_value);
      let param_value_length = match param_type {
        
        Some(pt) => {
          if let Some(len) = get_tv_param_length(pt) {
            len
          } else {
            warn!("Unknown TV parameter type: {}", param_type_value);
            continue;
          }
        }

        None => {
          warn!("Unknown TV parameter type: {}", param_type_value);
          continue;
        }
      };

      if buf.remaining() < param_value_length {
        return Err(Error::new(
          ErrorKind::InvalidData,
          "Buffer too short for TV parameter value"
        ));
      }

      let param_value = buf.split_to(param_value_length);

      let parameter = LlrpParameter {
        param_type: param_type.unwrap(),
        param_length: (1 + param_value_length) as u16,
        param_value: param_value.to_vec(),
        sub_params: None // TV parameters don't contain sub-parameters
      };

      parameters.push(parameter);

    } else {
      
      debug!("Parsing TLV parameter");

      if buf.remaining() < 4 {
        return Err(Error::new(
          ErrorKind::InvalidData,
          "Buffer too short for TLV parameter header"
        ));
      }

      let next_four_bytes = &buf[..4];
      debug!("Next 4 bytes: {:?}", next_four_bytes);
      debug!("Next 4 bytes (hex): {:02X?}", next_four_bytes);

      let param_type_value = buf.get_u16();
      let param_length = buf.get_u16();

      debug!("param_type_value: {}", param_type_value);
      debug!("param_length: {}", param_length);
      debug!("buf.remaining(): {:?}", buf.remaining());

      if param_length < 4 || (param_length - 4) as usize > buf.remaining() {
        
        debug!(
          "Invalid TLV parameter length:\n\tparam_length={}\n\tbuf.remaining()={}", 
          param_length, 
          buf.remaining()
        );

        debug!("Bytes at error point: {:02X?}", &buf[..]);
        
        return Err(Error::new(
          ErrorKind::InvalidData,
          "Invalid TLV parameter length"
        ));
      }

      let param_value_length = param_length as usize - 4;
      debug!("param_value_length: {}", param_value_length);

      let mut param_value = buf.split_to(param_value_length);

      let sub_params = if is_known_parameter_with_subparams(param_type_value) {
        Some(parse_parameters(&mut param_value)?)
      } else { None };

      let param_type = LlrpParameterType::from_value(param_type_value);
      if let Some(param_type) = param_type {
        
        let parameter = LlrpParameter {
          param_type,
          param_length,
          param_value: param_value.to_vec(),
          sub_params
        };

        debug!("Parsed parameter: {:?}", parameter.param_type);
        parameters.push(parameter);
      } else {
        warn!("Unknown TLV parameter type: {}", param_type_value);
      }
    }
  }

  Ok(parameters)
}

#[derive(Debug)]
pub struct GeneralDeviceCapabilities {
  pub max_number_of_antennas_supported  : u16,
  pub can_set_antenna_properties        : bool,
  pub has_utc_clock_capabilities        : bool,
  pub device_manufacturer_name          : u32,
  pub model_name                        : u32,
  pub reader_firmware_version           : String,
  pub receive_sensitivity_table_entries : Vec<ReceiveSensitivityTableEntry>,
  pub gpio_capabilities                 : Option<GPIOCapabilities>,
  pub antenna_air_protocols             : Vec<AntennaAirProtocol>
}

impl GeneralDeviceCapabilities {
  pub fn decode(
    buf: &[u8]
  ) -> io::Result<Self> {
    let mut buf = BytesMut::from(buf);

    if buf.remaining() < 9 {
      return Err(Error::new(
        ErrorKind::InvalidData,
        "Buffer too short for GeneralDeviceCapabilities"
      ));
    }

    let max_number_of_antennas_supported = buf.get_u16();

    let capabilities = buf.get_u16();
    let can_set_antenna_properties = (capabilities & 0x8000) != 0;
    let has_utc_clock_capabilities = (capabilities & 0x4000) != 0;

    let device_manufacturer_name = buf.get_u32();
    let model_name = buf.get_u32();

    let mut firmware_bytes = Vec::new();
    while buf.remaining() > 0 {
      let byte = buf.get_u8();
      if byte == 0 { break };
      firmware_bytes.push(byte);
    }

    let reader_firmware_version = String::from_utf8(firmware_bytes)
      .map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?;

    let sub_parameters = parse_parameters(&mut buf)?;

    let mut receive_sensitivity_table_entries = Vec::new();
    let mut gpio_capabilities = None;
    let mut antenna_air_protocols = Vec::new();

    for param in sub_parameters {
      match param.param_type {

        LlrpParameterType::ReceiveSensitivityTableEntry => {
          let entry = ReceiveSensitivityTableEntry::decode(&param.param_value)?;
          receive_sensitivity_table_entries.push(entry);
        }

        LlrpParameterType::GPIPCapabilities => {
          let gpio_caps = GPIOCapabilities::decode(&param.param_value)?;
          gpio_capabilities = Some(gpio_caps);
        }

        LlrpParameterType::PerAntennaAirProtocol => {
          let antenna_protocol = AntennaAirProtocol::decode(&param.param_value)?;
          antenna_air_protocols.push(antenna_protocol);
        }

        _ => {
          warn!("Unhandled sub-parameter type in GeneralDeviceCapabilities: {:?}", param.param_type);
        }
      }
    }

    Ok(GeneralDeviceCapabilities {
      max_number_of_antennas_supported,
      can_set_antenna_properties,
      has_utc_clock_capabilities,
      device_manufacturer_name,
      model_name,
      reader_firmware_version,
      receive_sensitivity_table_entries,
      gpio_capabilities,
      antenna_air_protocols
    })
  }
}

#[derive(Debug)]
pub struct GPIOCapabilities {
  pub num_gpi_ports : u16,
  pub num_gpo_ports : u16 
}

impl GPIOCapabilities {
  pub fn decode(
    buf: &[u8]
  ) -> io::Result<Self> {
    let mut buf = BytesMut::from(buf);

    if buf.remaining() < 4 {
      return Err(Error::new(
        ErrorKind::InvalidData,
        "Buffer too short for GPIOCapabilities"
      ));
    }

    let num_gpi_ports = buf.get_u16();
    let num_gpo_ports = buf.get_u16();

    Ok(GPIOCapabilities { 
      num_gpi_ports,
      num_gpo_ports
    })
  }
}

#[derive(Debug)]
pub struct AntennaAirProtocol {
  pub antenna_id   : u16,
  pub protocol_ids : Vec<u8>
}

impl AntennaAirProtocol {
  pub fn decode(
    buf: &[u8]
  ) -> io::Result<Self> {
    let mut buf = BytesMut::from(buf);

    if buf.remaining() < 3 {
      return Err(Error::new(
        ErrorKind::InvalidData,
        "Buffer too short for AntennaAirProtocol"
      ));
    }

    let antenna_id = buf.get_u16();
    let num_protocols = buf.get_u8();

    let mut protocol_ids = Vec::new();
    for _ in 0..num_protocols {
      if buf.remaining() < 1 {
        return Err(Error::new(
          ErrorKind::InvalidData,
          "Buffer too short for antenna air protocol IDs"
        ));
      }

      let protocol_id = buf.get_u8();
      protocol_ids.push(protocol_id);
    }

    Ok(AntennaAirProtocol {
      antenna_id,
      protocol_ids
    })
  }
}

#[derive(Debug)]
pub struct LLRPCapabilities {
  pub can_do_rfsurvey                               : bool,
  pub can_report_buffer_fill_warning                : bool,
  pub supports_client_request_op_spec               : bool,
  pub can_do_tag_inventory_state_aware_singulation  : bool,
  pub supports_event_and_report_holding             : bool,
  pub max_num_priority_levels_supported             : u8,
  pub client_request_op_spec_timeout                : u32,
  pub max_num_ro_specs                              : u32,
  pub max_num_specs_per_ro_spec                     : u32,
  pub max_num_inventory_parameter_specs_per_ai_spec : u32,
  pub max_num_access_specs                          : u32,
  pub max_num_op_specs_per_access_spec              : u32
}

impl LLRPCapabilities {
  pub fn decode(
    buf: &[u8]
  ) -> io::Result<Self> {
    let mut buf = BytesMut::from(buf);

    if buf.remaining() < 24 {
      return Err(Error::new(
        ErrorKind::InvalidData,
        "Buffer too short for LLRPCapabilities"
      ));
    }

    let capabilities = buf.get_u8();
    let can_do_rfsurvey                              = (capabilities & 0x80) != 0;
    let can_report_buffer_fill_warning               = (capabilities & 0x40) != 0;
    let supports_client_request_op_spec              = (capabilities & 0x20) != 0;
    let can_do_tag_inventory_state_aware_singulation = (capabilities & 0x10) != 0;
    let supports_event_and_report_holding            = (capabilities & 0x08) != 0;

    let max_num_priority_levels_supported               = buf.get_u8();
    let client_request_op_spec_timeout                 = buf.get_u32();
    let max_num_ro_specs                               = buf.get_u32();
    let max_num_specs_per_ro_spec                      = buf.get_u32();
    let max_num_inventory_parameter_specs_per_ai_spec  = buf.get_u32();
    let max_num_access_specs                           = buf.get_u32();
    let max_num_op_specs_per_access_spec               = buf.get_u32();

    Ok(LLRPCapabilities {
      can_do_rfsurvey,
      can_report_buffer_fill_warning,
      supports_client_request_op_spec,
      can_do_tag_inventory_state_aware_singulation,
      supports_event_and_report_holding,
      max_num_priority_levels_supported,
      client_request_op_spec_timeout,
      max_num_ro_specs,
      max_num_specs_per_ro_spec,
      max_num_inventory_parameter_specs_per_ai_spec,
      max_num_access_specs,
      max_num_op_specs_per_access_spec
    })
  }
}

#[derive(Debug)]
pub struct RegulatoryCapabilities {
  pub country_code            : u16,
  pub communications_standard : u8,
  pub uhf_band_capabilities   : Option<UHFBandCapabilities>
}

impl RegulatoryCapabilities {
  pub fn decode(
    buf: &[u8]
  ) -> io::Result<Self> {
    let mut buf = BytesMut::from(buf);

    if buf.remaining() < 3 {
      return Err(Error::new(
        ErrorKind::InvalidData,
        "Buffer too short for RegulatoryCapabilities"
      ));
    }

    let country_code = buf.get_u16();
    let communications_standard = buf.get_u8();

    let sub_parameters = parse_parameters(&mut buf)?;

    let mut uhf_band_capabilities = None;

    for param in sub_parameters {
      match param.param_type {
        
        LlrpParameterType::UHFBandCapabilities => {
          let uhf_caps = UHFBandCapabilities::decode(&param.param_value)?;
          uhf_band_capabilities = Some(uhf_caps);
        }

        _ => {
          warn!("Unhandled sub-parameter type in RegulatoryCapabilities: {:?}", param.param_type);
        }

      }      
    }

    Ok(RegulatoryCapabilities {
      country_code,
      communications_standard,
      uhf_band_capabilities
    })
  }
}

#[derive(Debug)]
pub struct UHFBandCapabilities {
  pub transmit_power_levels: Vec<TransmitPowerLevelTableEntry>,
  pub frequency_information: Option<FrequencyInformation>
}

impl UHFBandCapabilities {
  pub fn decode(
    buf: &[u8]
  ) -> io::Result<Self> {
    let mut buf = BytesMut::from(buf);
    let sub_parameters = parse_parameters(&mut buf)?;

    let mut transmit_power_levels = Vec::new();
    let mut frequency_information = None;

    for param in sub_parameters {
      match param.param_type {
        
        LlrpParameterType::TransmitPowerLevelTableEntry => {
          let entry = TransmitPowerLevelTableEntry::decode(&param.param_value)?;
          transmit_power_levels.push(entry);
        }

        LlrpParameterType::FrequencyInformation => {
          let freq_info = FrequencyInformation::decode(&param.param_value)?;
          frequency_information = Some(freq_info)
        }

        _ => {
          warn!("Unhandled sub-parameter type in UHFBandCapabilities: {:?}", param.param_type);
        }
      }
    }

    Ok(UHFBandCapabilities {
      transmit_power_levels,
      frequency_information
    })
  }
}

#[derive(Debug)]
pub struct TransmitPowerLevelTableEntry {
  pub index: u16,
  pub transmit_power_value: u16
}

impl TransmitPowerLevelTableEntry {
  pub fn decode(
    buf: &[u8]
  ) -> io::Result<Self> {
    let mut buf = BytesMut::from(buf);

    if buf.remaining() < 4 {
      return Err(Error::new(
        ErrorKind::InvalidData,
        "Buffer too short for TransmitPowerLevelTableEntry"
      ));
    }

    let index = buf.get_u16();
    let transmit_power_value = buf.get_u16();

    Ok(TransmitPowerLevelTableEntry {
      index,
      transmit_power_value
    })
  }
}

#[derive(Debug)]
pub struct ReceiveSensitivityTableEntry {
  pub index: u16,
  pub receive_sensitivity_value: i16
  //pub receive_sensitivity_value: u8
}

impl ReceiveSensitivityTableEntry {
  pub fn decode(
    buf: &[u8]
  ) -> io::Result<Self> {
    let mut buf = BytesMut::from(buf);

    if buf.remaining() < 4 {
      return Err(Error::new(
        ErrorKind::InvalidData,
        "Buffer too short for ReceiveSensitivityTableEntry"
      ));
    }

    let index = buf.get_u16();
    let receive_sensitivity_value = buf.get_i16();
    //let receive_sensitivity_value = buf.get_u8();

    Ok(ReceiveSensitivityTableEntry {
      index,
      receive_sensitivity_value
    })
  }
}

#[derive(Debug)]
pub struct FrequencyInformation {
  pub hopping               : bool,
  pub frequency_hop_tables  : Vec<FrequencyHopTable>,
  pub fixed_frequency_table : Option<FixedFrequencyTable>
}

impl FrequencyInformation {
  pub fn decode(
    buf: &[u8]
  ) -> io::Result<Self> {
    let mut buf = BytesMut::from(buf);

    if buf.remaining() < 1 {
      return Err(Error::new(
        ErrorKind::InvalidData,
        "Buffer too short for FrequencyInformation"
      ));
    }

    let hop_flag = buf.get_u8();
    let hopping = hop_flag != 0;

    let sub_parameters = parse_parameters(&mut buf)?;

    let mut frequency_hop_tables = Vec::new();
    let mut fixed_frequency_table = None;

    for param in sub_parameters {
      match param.param_type {

        LlrpParameterType::FrequencyHopTable => {
          let hop_table = FrequencyHopTable::decode(&param.param_value)?;
          frequency_hop_tables.push(hop_table);
        }

        LlrpParameterType::FixedFrequencyTable => {
          fixed_frequency_table = Some(FixedFrequencyTable::decode(&param.param_value)?);
        }

        _ => {
          warn!("Unhandled sub_parameter type in FrequencyInformation: {:?}", param.param_type);
        }
      }
    }

    Ok(FrequencyInformation {
      hopping,
      frequency_hop_tables,
      fixed_frequency_table
    })
  }
}

#[derive(Debug)]
pub struct FrequencyHopTable {
  pub hop_table_id: u16,
  pub frequencies: Vec<u32>
}

impl FrequencyHopTable {
  pub fn decode(
    buf: &[u8]
  ) -> io::Result<Self> {
    let mut buf = BytesMut::from(buf);

    if buf.remaining() < 4 {
      return Err(Error::new(
        ErrorKind::InvalidData,
        "Buffer too short for FrequencyHopTable header"
      ));
    }

    let hop_table_id = buf.get_u16();
    let num_frequencies = buf.get_u16();

    let frequencies_size = num_frequencies as usize * 4;

    if buf.remaining() < frequencies_size {
      return Err(Error::new(
        ErrorKind::InvalidData,
        "Buffer too short for FrequencyHopTable frequencies"
      ));
    }

    let mut frequencies = Vec::with_capacity(num_frequencies as usize);
    for _ in 0..num_frequencies {
      let frequency = buf.get_u32();
      frequencies.push(frequency);
    }

    Ok(FrequencyHopTable {
      hop_table_id,
      frequencies
    })
  }
}

#[derive(Debug)]
pub struct FixedFrequencyTable {
  pub frequencies: Vec<u32>
}

impl FixedFrequencyTable {
  pub fn decode(
    buf: &[u8]
  ) -> io::Result<Self> {
    let mut buf = BytesMut::from(buf);

    if buf.remaining() < 2 {
      return Err(Error::new(
        ErrorKind::InvalidData, 
        "Buffer too short for FixedFrequencyTable header"
      ));
    }

    let num_frequencies = buf.get_u16();
    let frequencies_size = num_frequencies as usize * 4;

    if buf.remaining() < frequencies_size {
      return Err(Error::new(
        ErrorKind::InvalidData,
        "Buffer too short for FixedFrequencyTable frequencies"
      ));
    }

    let mut frequencies = Vec::with_capacity(num_frequencies as usize);
    for _ in 0..num_frequencies {
      let frequency = buf.get_u32();
      frequencies.push(frequency);
    }

    Ok(FixedFrequencyTable { frequencies })
  }
}

#[derive(Debug)]
pub struct TagReportData {
  pub epc: Vec<u8>
}

impl fmt::Display for TagReportData {
  fn fmt(
    &self, 
    f: &mut fmt::Formatter<'_>
  ) -> fmt::Result {
    
    let epc_hex = self.epc.iter()
      .map(|byte| format!("{:02x}", byte))
      .collect::<Vec<String>>()
      .join("");

    write!(f, "{}", epc_hex)
  }
}

impl TagReportData {
  
  pub fn decode(
    buf: &[u8]
  ) -> io::Result<Self> {

    let mut buf = BytesMut::from(buf);
    let mut epc = Vec::new();

    let parameters = parse_parameters(&mut buf)?;

    for parameter in parameters {
      match parameter.param_type {

        LlrpParameterType::EPCData => {
          let epc_data = EPCData::decode(&parameter.param_value)?;
          epc = epc_data.epc;
        }

        LlrpParameterType::EPC96 => {
          let epc_data = EPCData::decode_epc96(&parameter.param_value)?;
          epc = epc_data.epc;
        }

        _ => {
          warn!("Unhandled sub-parameter type: {:?}", parameter.param_type);
        }
      }
    }

    Ok(TagReportData { epc })
  }
}

#[derive(Debug)]
pub struct EPCData {
  pub epc: Vec<u8>
}

impl fmt::Display for EPCData {
  fn fmt(
    &self, 
    f: &mut fmt::Formatter<'_>
  ) -> fmt::Result {
    
    let epc_hex = self.epc.iter()
      .map(|byte| format!("{:02x}", byte))
      .collect::<Vec<String>>()
      .join("");

    write!(f, "{}", epc_hex)
  }
}

impl EPCData {
  
  pub fn decode(
    buf: &[u8]
  ) -> io::Result<Self> {

    let mut buf = BytesMut::from(buf);

    if buf.remaining() < 2 {
      return Err(Error::new(
        ErrorKind::InvalidData,
        "Buffer too short for EPCData Bit Field Length"
      ));
    }

    let bit_field_length = buf.get_u16();
    let epc_byte_length = ((bit_field_length + 7) / 8) as usize;

    if buf.remaining() < epc_byte_length {
      return Err(Error::new(
        ErrorKind::InvalidData,
        "Buffer too short for EPCData EPC field"
      ));
    }

    let epc = buf.split_to(epc_byte_length).to_vec();

    Ok(EPCData { epc })
  }

  pub fn decode_epc96(
    buf: &[u8]
  ) -> io::Result<Self> {

    if buf.len() != 12 {
      return Err(Error::new(
        ErrorKind::InvalidData,
        "EPC96 data must be 12 bytes",
      ));
    }

    let epc = buf.to_vec();
    Ok(EPCData { epc })
  }
}

fn is_known_parameter_with_subparams(param_type_value: u16) -> bool {
  match LlrpParameterType::from_value(param_type_value) {
    Some(LlrpParameterType::GeneralDeviceCapabilities) |
    Some(LlrpParameterType::LLRPCapabilities)          |
    Some(LlrpParameterType::RegulatoryCapabilities)    |
    Some(LlrpParameterType::UHFBandCapabilities)       | 
    Some(LlrpParameterType::FrequencyInformation) => true,
    _ => false
  }
}

fn get_tv_param_length(param_type: LlrpParameterType) -> Option<usize> {
  match param_type {
    LlrpParameterType::EPC96 => Some(12),
    _ => None
  }
}