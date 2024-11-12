use bytes::{BytesMut, Buf, BufMut};
use std::io::{self, Error, ErrorKind};

// Reader Operation (RO) Message Type Enum
pub const GET_READER_CAPABILITIES          : u16 = 1;
pub const GET_READER_CAPABILITIES_RESPONSE : u16 = 11;
pub const GET_READER_CONFIG                : u16 = 2;
pub const GET_READER_CONFIG_RESPONSE       : u16 = 12;
pub const SET_READER_CONFIG                : u16 = 3;
pub const SET_READER_CONFIG_RESPONSE       : u16 = 13;
pub const CLOSE_CONNECTION                 : u16 = 14;
pub const CLOSE_CONNECTION_RESPONSE        : u16 = 4;
pub const ADD_ROSPEC                       : u16 = 20;
pub const ADD_ROSPEC_RESPONSE              : u16 = 30;
pub const DELETE_ROSPEC                    : u16 = 21;
pub const DELETE_ROSPEC_RESPONSE           : u16 = 31;
pub const START_ROSPEC                     : u16 = 22;
pub const START_ROSPEC_RESPONSE            : u16 = 32;
pub const STOP_ROSPEC                      : u16 = 23;
pub const STOP_ROSPEC_RESPONSE             : u16 = 33;
pub const ENABLE_ROSPEC                    : u16 = 24;
pub const ENABLE_ROSPEC_RESPONSE           : u16 = 34;
pub const DISABLE_ROSPEC                   : u16 = 25;
pub const DISABLE_ROSPEC_RESPONSE          : u16 = 35;
pub const GET_ROSPECS                      : u16 = 26;
pub const GET_ROSPECS_RESPONSE             : u16 = 36;
pub const GET_REPORT                       : u16 = 60;
pub const RO_ACCESS_REPORT                 : u16 = 61;
pub const KEEPALIVE                        : u16 = 62;
pub const KEEPALIVE_ACK                    : u16 = 72;
pub const READER_EVENT_NOTIFICATION        : u16 = 63;
pub const ENABLE_EVENTS_AND_REPORTS        : u16 = 64;
pub const ERROR_MESSAGE                    : u16 = 100;
pub const CUSTOM_MESSAGE                   : u16 = 1023;

// Params
pub const PARAM_UTC_TIME_STAMP                        : u16 = 128;
pub const PARAM_UPTIME                                : u16 = 129;
pub const PARAM_GENERAL_DEVICE_CAPABILITIES           : u16 = 137;
pub const PARAM_MAXIMUM_RECEIVE_SENSITIVITY           : u16 = 363;
pub const PARAM_RECEIVE_SENSITIVITY_TABLE_ENTRY       : u16 = 139;
pub const PARAM_PER_ANTENNA_AIR_PROTOCOL              : u16 = 140;
pub const PARAM_GPIO_CAPABILITIES                     : u16 = 141;
pub const PARAM_LLRP_CAPABILITIES                     : u16 = 142;
pub const PARAM_REGULATORY_CAPABILITIES               : u16 = 143;
pub const PARAM_UHF_BAND_CAPABILITIES                 : u16 = 144;
pub const PARAM_TRANSMIT_POWER_LEVEL_TABLE_ENTRY      : u16 = 145;
pub const PARAM_FREQUENCY_INFORMATION                 : u16 = 146;
pub const PARAM_FREQUENCY_HOP_TABLE                   : u16 = 147;
pub const PARAM_FIXED_FREQUENCY_TABLE                 : u16 = 148;
pub const PARAM_PER_ANTENNA_RECEIVE_SENSITIVITY_RANGE : u16 = 149;
pub const PARAM_RF_SURVEY_FREQUENCY_CAPABILITIES      : u16 = 365;
pub const PARAM_RO_SPEC                               : u16 = 177;
pub const PARAM_RO_BOUNDARY_SPEC                      : u16 = 178;
pub const PARAM_RO_SPEC_START_TRIGGER                 : u16 = 179;
pub const PARAM_PERIODIC_TRIGGER_VALUE                : u16 = 180;
pub const PARAM_GPI_TRIGGER_VALUE                     : u16 = 181;
pub const PARAM_RO_SPEC_STOP_TRIGGER                  : u16 = 182;
pub const PARAM_AI_SPEC                               : u16 = 183;
pub const PARAM_AI_SPEC_STOP_TRIGGER                  : u16 = 184;
pub const PARAM_TAG_OBSERVATION_TRIGGER               : u16 = 185;
pub const PARAM_INVENTORY_PARAMETER_SPEC              : u16 = 186;
pub const PARAM_RF_SURVEY_SPEC                        : u16 = 187;
pub const PARAM_RF_SURVEY_SPEC_STOP_TRIGGER           : u16 = 188;
pub const PARAM_LOOP_SPEC                             : u16 = 355;
pub const PARAM_ACCESS_SPEC                           : u16 = 207;
pub const PARAM_ACCESS_SPEC_STOP_TRIGGER              : u16 = 208;
pub const PARAM_ACCESS_COMMAND                        : u16 = 209;
pub const PARAM_CLIENT_REQUEST_OP_SPEC                : u16 = 210;
pub const PARAM_CLIENT_REQUEST_RESPONSE               : u16 = 211;
pub const PARAM_LLRP_CONFIGURATION_STATE_VALUE        : u16 = 217;
pub const PARAM_IDENTIFICATION                        : u16 = 218;
pub const PARAM_GPO_WRITE_DATA                        : u16 = 219;
pub const PARAM_KEEP_ALIVE_SPEC                       : u16 = 220;
pub const PARAM_ANTENNA_PROPERTIES                    : u16 = 221;
pub const PARAM_ANTENNA_CONFIGURATION                 : u16 = 222;
pub const PARAM_RF_RECEIVER                           : u16 = 223;
pub const PARAM_RF_TRANSMITTER                        : u16 = 224;
pub const PARAM_GPI_PORT_CURRENT_STATE                : u16 = 225;
pub const PARAM_EVENTS_AND_REPORTS                    : u16 = 226;
pub const PARAM_RO_REPORT_SPEC                        : u16 = 237;
pub const PARAM_TAG_REPORT_CONTENT_SELECTOR           : u16 = 238;


#[derive(Debug)]
pub struct LlrpMessage {
  pub message_type   : u16,
  pub message_length : u32,
  pub message_id     : u32,
  pub payload        : Vec<u8>
}

impl LlrpMessage {

  pub fn new(message_type: u16, message_id: u32, payload: Vec<u8>) -> Self {
    let message_length = 10 + payload.len() as u32;
    LlrpMessage {
      message_type,
      message_length,
      message_id,
      payload
    }
  }

  pub fn new_enable_events_and_reports(message_id: u32) -> Self {
    LlrpMessage::new(ENABLE_EVENTS_AND_REPORTS, message_id, vec![])
  }

  pub fn add_rospec(message_id: u32, rospec_id: u32) -> Self {
    let mut payload = BytesMut::with_capacity(32);

    // ROSpecID, Priority, and CurrentState
    payload.put_u16(PARAM_RO_SPEC);      // ROSpec Parameter Type (16 bits)
    payload.put_u16(10);                 // Length (16 bits)
    payload.put_u32(rospec_id);          // ROSpecID
    payload.put_u8(0);                   // Priority
    payload.put_u8(0);                   // CurrentState

    // ROBoundarySpec (StartTrigger and StopTrigger)
    payload.put_u16(PARAM_RO_BOUNDARY_SPEC);  // ROBoundarySpec Parameter Type (16 bits)
    payload.put_u16(8);                       // Length (16 bits)
    payload.put_u8(0);                        // StartTriggerType (Immediate)
    payload.put_u8(2);                        // StopTriggerType (Duration)
    payload.put_u32(1000);                    // StopTrigger duration (milliseconds)

    // AISpec
    payload.put_u16(PARAM_AI_SPEC);           // AISpec Parameter Type (16 bits)
    payload.put_u16(10);                      // Length (16 bits)
    payload.put_u16(1);                       // AntennaID
    payload.put_u16(0);                       // InventoryParameterSpecID
    payload.put_u8(0);                        // AISpecStopTriggerType
    payload.put_u32(100);                     // AISpecStopTrigger duration

    // ROReportSpec
    payload.put_u16(PARAM_RO_REPORT_SPEC);    // ROReportSpec Parameter Type (16 bits)
    payload.put_u16(6);                       // Length (16 bits)
    payload.put_u8(1);                        // ReportTrigger (End of ROSpec)
    payload.put_u8(1);                        // ReportContentSelector (TagInfo/EPC)

    LlrpMessage::new(ADD_ROSPEC, message_id, payload.to_vec())
  }


  /*
  pub fn add_rospec(message_id: u32, rospec_id: u32) -> Self {
    let mut payload = BytesMut::with_capacity(8);

    // ROSpec parameters
    payload.put_u32(rospec_id); // ROSpecID
    payload.put_u8(0);          // Priority (0 = Default)
    payload.put_u8(0);          // CurrentState (0 = Disabled)

    // ROBoundarySpec (Defines the start and stop conditions for the ROSpec)
    payload.put_u16(1);         // ROBoundarySpec type (custom identifier)
    payload.put_u8(0);          // StartTriggerType (0 = Immediate)
    //payload.put_u8(0);        // StopTriggerType (0 = Null - runs indefinitely)
    payload.put_u8(2);          // StopTriggerType (2 = Duration)
    payload.put_u32(1000);      // StopTrigger duration in milliseconds

    // AISpec (Air Interface)
    payload.put_u16(1);         // AISpec type
    payload.put_u32(1);         // AntennaCount
    //payload.put_u16(0);       // InventoryParameterSpec ID (0 = Default)
    payload.put_u16(1);         // InventoryParameterSpec ID (1 = EPC Gen2)

    // ROReportSpec
    payload.put_u16(1);         // ROReportSpec type (custom identifier)
    payload.put_u8(1);          // ReportTrigger (0 = None, 1 = end of ROSpec)
    payload.put_u8(1);          // ReportContentSelector (1 = report TagInfo/EPC)

    LlrpMessage::new(ADD_ROSPEC, message_id, payload.to_vec())
  }
  */

  pub fn new_start_rospec(message_id: u32, rospec_id: u32) -> Self {
    let mut payload = BytesMut::with_capacity(4);
    payload.put_u32(rospec_id);
    LlrpMessage::new(START_ROSPEC, message_id, payload.to_vec())
  }

  pub fn new_stop_rospec(message_id: u32, rospec_id: u32) -> Self {
    let mut payload = BytesMut::with_capacity(4);
    payload.put_u32(rospec_id);
    LlrpMessage::new(STOP_ROSPEC, message_id, payload.to_vec())
  }

  pub fn new_delete_rospec(message_id: u32, rospec_id: u32) -> Self {
    let mut payload = BytesMut::with_capacity(4);
    payload.put_u32(rospec_id);
    LlrpMessage::new(DELETE_ROSPEC, message_id, payload.to_vec())
  }

  pub fn new_close_connection(message_id: u32) -> Self {
    LlrpMessage::new(CLOSE_CONNECTION , message_id, vec![])
  }

  pub fn encode(&self) -> BytesMut {
    let mut buffer = BytesMut::with_capacity(self.message_length as usize);

    let version = 1;
    let reserved = 0;
    let version_and_type = ((version & 0x7) << 13) | ((reserved & 0x7) << 10) | (self.message_type & 0x3FFF);

    buffer.put_u16(version_and_type as u16);
    buffer.put_u32(self.message_length);
    buffer.put_u32(self.message_id);
    buffer.extend_from_slice(&self.payload);

    buffer
  }

  /*
  pub fn encode(&self) -> BytesMut {
    let mut buffer = BytesMut::with_capacity(self.message_length as usize);

    let version = 1; // Assuming LLRP version 1.0
    let version_and_type = ((version & 0x7) << 10) | (self.message_type & 0x3FF);

    buffer.put_u16(version_and_type as u16);
    buffer.put_u32(self.message_length);
    buffer.put_u32(self.message_id);
    buffer.extend_from_slice(&self.payload);

    buffer
  }
  */

  pub fn decode(buf: &mut BytesMut) -> io::Result<Self> {
    if buf.len() < 10 {
      return Err(Error::new(ErrorKind::InvalidData, "Buffer too short for LLRP header"));
    }

    let version_and_type = buf.get_u16();
    let version = (version_and_type >> 10) & 0x7;
    let message_type = version_and_type & 0x3FF;
    let message_length = buf.get_u32();
    let message_id = buf.get_u32();

    if buf.len() < (message_length - 10) as usize {
      return Err(Error::new(ErrorKind::InvalidData, "Buffer too short for payload"));
    }

    let payload = buf.split_to((message_length - 10) as usize).to_vec();

    Ok(LlrpMessage {
      message_type,
      message_length,
      message_id,
      payload,
    })
  }
}

pub struct TagReport {
  pub epc: Vec<u8>, // EPC (Electronic Product Code) data
  pub timestamp: u64,
}

impl TagReport {

  pub fn decode(buf: &mut BytesMut) -> io::Result<Self> {
    if buf.len() < 10 {
      return Err(Error::new(ErrorKind::InvalidData, "Buffer too short for Tag Report"));
    }

    let timestamp    = buf.get_u64();
    let epc_length = buf.get_u8() as usize;
    let epc      = buf.split_to(epc_length).to_vec();

    Ok(TagReport { epc, timestamp })
  }
}