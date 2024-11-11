use bytes::{BytesMut, Buf, BufMut};
use std::{arch::x86_64::_CMP_TRUE_US, io::{self, Error, ErrorKind}};

// Configuration Type Enums
pub const GET_SUPPORTED_VERSION            : u16 = 46;
pub const GET_SUPPORTED_VERSION_RESPONSE   : u16 = 56;
pub const SET_PROTOCOL_VERSION             : u16 = 47;
pub const SET_PROTOCOL_VERSION_RESPONSE    : u16 = 57;
pub const GET_READER_CAPABILITIES          : u16 = 1;
pub const GET_READER_CAPABILITIES_RESPONSE : u16 = 11;
pub const GET_READER_CONFIG                : u16 = 2;
pub const GET_READER_CONFIG_RESPONSE       : u16 = 12;
pub const SET_READER_CONFIG                : u16 = 3;
pub const SET_READER_CONFIG_RESPONSE       : u16 = 13;

// Reader Operation (RO) Message Type Enums
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

// Params Enums
pub const PARAM_UTC_TIME_STAMP                                    : u16 = 128;
pub const PARAM_UPTIME                                            : u16 = 129;
pub const PARAM_GENERAL_DEVICE_CAPABILITIES                       : u16 = 137;
pub const PARAM_MAXIMUM_RECEIVE_SENSITIVITY                       : u16 = 363;
pub const PARAM_RECEIVE_SENSITIVITY_TABLE_ENTRY                   : u16 = 139;
pub const PARAM_PER_ANTENNA_AIR_PROTOCOL                          : u16 = 140;
pub const PARAM_GPIO_CAPABILITIES                                 : u16 = 141;
pub const PARAM_LLRP_CAPABILITIES                                 : u16 = 142;
pub const PARAM_REGULATORY_CAPABILITIES                           : u16 = 143;
pub const PARAM_UHF_BAND_CAPABILITIES                             : u16 = 144;
pub const PARAM_TRANSMIT_POWER_LEVEL_TABLE_ENTRY                  : u16 = 145;
pub const PARAM_FREQUENCY_INFORMATION                             : u16 = 146;
pub const PARAM_FREQUENCY_HOP_TABLE                               : u16 = 147;
pub const PARAM_FIXED_FREQUENCY_TABLE                             : u16 = 148;
pub const PARAM_PER_ANTENNA_RECEIVE_SENSITIVITY_RANGE             : u16 = 149;
pub const PARAM_RF_SURVEY_FREQUENCY_CAPABILITIES                  : u16 = 365;
pub const PARAM_RO_SPEC                                           : u16 = 177;
pub const PARAM_RO_BOUNDARY_SPEC                                  : u16 = 178;
pub const PARAM_RO_SPEC_START_TRIGGER                             : u16 = 179;
pub const PARAM_PERIODIC_TRIGGER_VALUE                            : u16 = 180;
pub const PARAM_GPI_TRIGGER_VALUE                                 : u16 = 181;
pub const PARAM_RO_SPEC_STOP_TRIGGER                              : u16 = 182;
pub const PARAM_AI_SPEC                                           : u16 = 183;
pub const PARAM_AI_SPEC_STOP_TRIGGER                              : u16 = 184;
pub const PARAM_TAG_OBSERVATION_TRIGGER                           : u16 = 185;
pub const PARAM_INVENTORY_PARAMETER_SPEC                          : u16 = 186;
pub const PARAM_RF_SURVEY_SPEC                                    : u16 = 187;
pub const PARAM_RF_SURVEY_SPEC_STOP_TRIGGER                       : u16 = 188;
pub const PARAM_LOOP_SPEC                                         : u16 = 355;
pub const PARAM_ACCESS_SPEC                                       : u16 = 207;
pub const PARAM_ACCESS_SPEC_STOP_TRIGGER                          : u16 = 208;
pub const PARAM_ACCESS_COMMAND                                    : u16 = 209;
pub const PARAM_CLIENT_REQUEST_OP_SPEC                            : u16 = 210;
pub const PARAM_CLIENT_REQUEST_RESPONSE                           : u16 = 211;
pub const PARAM_LLRP_CONFIGURATION_STATE_VALUE                    : u16 = 217;
pub const PARAM_IDENTIFICATION                                    : u16 = 218;
pub const PARAM_GPO_WRITE_DATA                                    : u16 = 219;
pub const PARAM_KEEP_ALIVE_SPEC                                   : u16 = 220;
pub const PARAM_ANTENNA_PROPERTIES                                : u16 = 221;
pub const PARAM_ANTENNA_CONFIGURATION                             : u16 = 222;
pub const PARAM_RF_RECEIVER                                       : u16 = 223;
pub const PARAM_RF_TRANSMITTER                                    : u16 = 224;
pub const PARAM_GPI_PORT_CURRENT_STATE                            : u16 = 225;
pub const PARAM_EVENTS_AND_REPORTS                                : u16 = 226;
pub const PARAM_RO_REPORT_SPEC                                    : u16 = 237;
pub const PARAM_TAG_REPORT_CONTENT_SELECTOR                       : u16 = 238;
pub const PARAM_ACCESS_REPORT_SPEC                                : u16 = 239;
pub const PARAM_TAG_REPORT_DATA                                   : u16 = 240;
pub const PARAM_EPC_DATA                                          : u16 = 241;
pub const PARAM_EPC_96                                            : u16 = 13;
pub const PARAM_ROSPEC_ID                                         : u16 = 9;
pub const PARAM_SPEC_INDEX                                        : u16 = 14;
pub const PARAM_INVENTORY_PARAMETER_SPEC_ID                       : u16 = 10;
pub const PARAM_ANTENNA_ID                                        : u16 = 1;
pub const PARAM_PEAK_RSSI                                         : u16 = 6;
pub const PARAM_CHANNEL_INDEX                                     : u16 = 7;
pub const PARAM_FIRST_SEEN_TIMESTAMP_UTC                          : u16 = 2;
pub const PARAM_FIRST_SEEN_TIMESTAMP_UPTIME                       : u16 = 3;
pub const PARAM_LAST_SEEN_TIMESTAMP_UTC                           : u16 = 4;
pub const PARAM_LAST_SEEN_TIMESTAMP_UPTIME                        : u16 = 5;
pub const PARAM_TAG_SEEN_COUNT                                    : u16 = 8;
pub const PARAM_CLIENT_REQUEST_OP_SPEC_RESULT                     : u16 = 15;
pub const PARAM_ACCESS_SPEC_ID                                    : u16 = 16;
pub const PARAM_RF_SURVEY_REPORT_DATA                             : u16 = 242;
pub const PARAM_FREQUENCY_RSSI_LEVEL_ENTRY                        : u16 = 243;
pub const PARAM_READER_EVENT_NOTIFICATION_SPEC                    : u16 = 244;
pub const PARAM_EVENT_NOTIFICATION_STATE                          : u16 = 245;
pub const PARAM_READER_EVENT_NOTIFICATION_DATA                    : u16 = 246;
pub const PARAM_HOPPING_EVENT                                     : u16 = 247;
pub const PARAM_GPI_EVENT                                         : u16 = 248;
pub const PARAM_ROSPEC_EVENT                                      : u16 = 249;
pub const PARAM_REPORT_BUFFER_LEVEL_WARNING_EVENT                 : u16 = 250;
pub const PARAM_REPORT_BUFFER_OVERFLOW_ERROR_EVENT                : u16 = 251;
pub const PARAM_READER_EXCEPTION_EVENT                            : u16 = 252;
pub const PARAM_OP_SPEC_ID                                        : u16 = 17;
pub const PARAM_RF_SURVEY_EVENT                                   : u16 = 253;
pub const PARAM_AISPEC_EVENT                                      : u16 = 254;
pub const PARAM_ANTENNA_EVENT                                     : u16 = 255;
pub const PARAM_CONNECTION_ATTEMPT_EVENT                          : u16 = 256;
pub const PARAM_CONNECTION_CLOSE_EVENT                            : u16 = 257;
pub const PARAM_SPEC_LOOP_EVENT                                   : u16 = 356;
pub const PARAM_LLRP_STATUS                                       : u16 = 287;
pub const PARAM_FIELD_ERROR                                       : u16 = 288;
pub const PARAM_PARAMETER_ERROR                                   : u16 = 289;
pub const PARAM_CRYPTO_RESPONSE                                   : u16 = 290;
pub const PARAM_CUSTOM                                            : u16 = 1023;
pub const PARAM_C1G2_LLRP_CAPABILITIES                            : u16 = 327;
pub const PARAM_UHF_C1G2_RF_MODE_TABLE                            : u16 = 328;
pub const PARAM_UHF_C1G2_RF_MODE_TABLE_ENTRY                      : u16 = 329;
pub const PARAM_C1G2_INVENTORY_COMMAND                            : u16 = 330;
pub const PARAM_C1G2_FILTER                                       : u16 = 331;
pub const PARAM_C1G2_TAG_INVENTORY_MASK                           : u16 = 332;
pub const PARAM_C1G2_TAG_INVENTORY_STATE_AWARE_FILTER_ACTION      : u16 = 333;
pub const PARAM_C1G2_TAG_INVENTORY_STATE_UNAWARE_FILTER_ACTION    : u16 = 334;
pub const PARAM_C1G2_RF_CONTROL                                   : u16 = 335;
pub const PARAM_C1G2_SINGULATION_CONTROL                          : u16 = 336;
pub const PARAM_C1G2_TAG_INVENTORY_STATE_AWARE_SINGULATION_ACTION : u16 = 337;
pub const PARAM_C1G2_TAG_SPEC                                     : u16 = 338;
pub const PARAM_C1G2_TARGET_TAG                                   : u16 = 339;
pub const PARAM_C1G2_READ                                         : u16 = 341;
pub const PARAM_C1G2_WRITE                                        : u16 = 342;
pub const PARAM_C1G2_KILL                                         : u16 = 343;
pub const PARAM_RESERVED                                          : u16 = 357;
pub const PARAM_C1G2_LOCK                                         : u16 = 344;
pub const PARAM_C1G2_LOCK_PAYLOAD                                 : u16 = 345;
pub const PARAM_C1G2_BLOCK_ERASE                                  : u16 = 346;
pub const PARAM_C1G2_BLOCK_WRITE                                  : u16 = 347;
pub const PARAM_C1G2_BLOCK_PERMALOCK                              : u16 = 358;
pub const PARAM_C1G2_GET_BLOCK_PERMALOCK_STATUS                   : u16 = 359;
pub const PARAM_C1G2_EPC_MEMORY_SELECTOR                          : u16 = 348;
pub const PARAM_C1G2_PC                                           : u16 = 12;
pub const PARAM_C1G2_XPCW1                                        : u16 = 19;
pub const PARAM_C1G2_XPCW2                                        : u16 = 20;
pub const PARAM_C1G2_CRC                                          : u16 = 11;
pub const PARAM_C1G2_SINGULATION_DETAILS                          : u16 = 18;
pub const PARAM_C1G2_READ_OP_SPEC_RESULT                          : u16 = 349;
pub const PARAM_C1G2_WRITE_OP_SPEC_RESULT                         : u16 = 350;
pub const PARAM_C1G2_KILL_OP_SPEC_RESULT                          : u16 = 351;
pub const PARAM_RESERVED_360                                      : u16 = 360;
pub const PARAM_C1G2_LOCK_OP_SPEC_RESULT                          : u16 = 352;
pub const PARAM_C1G2_BLOCK_ERASE_OP_SPEC_RESULT                   : u16 = 353;
pub const PARAM_C1G2_CHALLENGE                                    : u16 = 366;
pub const PARAM_C1G2_BLOCK_WRITE_OP_SPEC_RESULT                   : u16 = 354;
pub const PARAM_C1G2_BLOCK_PERMALOCK_OP_SPEC_RESULT               : u16 = 361;
pub const PARAM_C1G2_GET_BLOCK_PERMALOCK_STATUS_OP_SPEC_RESULT    : u16 = 362;
pub const PARAM_C1G2_UNTRACEABLE                                  : u16 = 380;
pub const PARAM_C1G2_UNTRACEABLE_OP_SPEC_RESULT                   : u16 = 364;
pub const PARAM_C1G2_AUTHENTICATE                                 : u16 = 367;
pub const PARAM_C1G2_AUTH_COMM                                    : u16 = 368;
pub const PARAM_C1G2_SECURE_COMM                                  : u16 = 369;
pub const PARAM_C1G2_READ_BUFFER                                  : u16 = 370;
pub const PARAM_C1G2_KEY_UPDATE                                   : u16 = 372;
pub const PARAM_C1G2_TAG_PRIVILEGE                                : u16 = 373;
pub const PARAM_C1G2_AUTHENTICATE_OP_SPEC_RESULT                  : u16 = 374;
pub const PARAM_C1G2_AUTH_COMM_OP_SPEC_RESULT                     : u16 = 375;
pub const PARAM_C1G2_SECURE_COMM_OP_SPEC_RESULT                   : u16 = 376;
pub const PARAM_C1G2_READ_BUFFER_OP_SPEC_RESULT                   : u16 = 377;
pub const PARAM_C1G2_KEY_UPDATE_OP_SPEC_RESULT                    : u16 = 378;
pub const PARAM_C1G2_TAG_PRIVILEGE_OP_SPEC_RESULT                 : u16 = 379;
pub const PARAM_EXTEND_ON_TIME                                    : u16 = 381;
pub const PARAM_RESERVED_ISO                                      : u16 = 900;

// ROSpecStartTriggerType Enums
pub const START_TRIGGER_NULL               : u8 = 0;
pub const START_TRIGGER_IMMEDIATE          : u8 = 1;
pub const START_TRIGGER_PERIODIC           : u8 = 2;
pub const START_TRIGGER_GPI                : u8 = 3;
pub const START_TRIGGER_TAG_OBSERVATION    : u8 = 4;
pub const START_TRIGGER_AISPEC_EVENT       : u8 = 5;
pub const START_TRIGGER_NETWORK_CONNECTION : u8 = 6;

// ROSpecStopTriggerType Enums
pub const STOP_TRIGGER_NULL                    : u8 = 0;
pub const STOP_TRIGGER_DURATION                : u8 = 1;
pub const STOP_TRIGGER_GPI_WITH_TIMEOUT        : u8 = 2;
pub const STOP_TRIGGER_TAG_OBSERVATION         : u8 = 3;
pub const STOP_TRIGGER_N_TAGS_OR_END_OF_AISPEC : u8 = 4;
pub const STOP_TRIGGER_NETWORK_CONNECTION      : u8 = 5;

// AISpecStopTriggerType Enums
pub const AI_STOP_TRIGGER_NULL             : u8 = 0;
pub const AI_STOP_TRIGGER_DURATION         : u8 = 1;
pub const AI_STOP_TRIGGER_GPI_WITH_TIMEOUT : u8 = 2;
pub const AI_STOP_TRIGGER_TAG_OBSERVATION  : u8 = 3;
pub const AI_STOP_TRIGGER_N_ATTEMPTS       : u8 = 4;

// RFID Tag Protocol Enums
pub const PROTOCOL_ID_UNSPECIFIED            : u16 = 0;
pub const PROTOCOL_ID_EPC_GLOBAL_CLASS1_GEN2 : u16 = 1;
pub const PROTOCOL_ID_UBIQUITOUS_ID_CLASS1   : u16 = 2;
pub const PROTOCOL_ID_ISO180006B             : u16 = 3;
pub const PROTOCOL_ID_ISO180006A             : u16 = 4;
pub const PROTOCOL_ID_IPX64                  : u16 = 5;
pub const PROTOCOL_ID_IPX256                 : u16 = 6;
pub const PROTOCOL_ID_ISO180006C             : u16 = 7;


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
      payload: payload
    }
  }

  pub fn new_enable_events_and_reports(message_id: u32) -> Self {
    LlrpMessage::new(ENABLE_EVENTS_AND_REPORTS, message_id, vec![])
  }

  pub fn new_get_supported_version(message_id: u32) -> Self {
    LlrpMessage::new(GET_SUPPORTED_VERSION, message_id, vec![])
  }

  pub fn new_set_protocol_version(message_id: u32, version: u8) -> Self {
    let mut payload = BytesMut::new();
    payload.put_u8(version); // Protocol version
    payload.put_u8(0);       // Reserved
    
    // Padding to align to 8 bytes
    payload.put_slice(&[0u8; 6]);
    
    LlrpMessage::new(SET_PROTOCOL_VERSION, message_id, payload.to_vec())
  }

  pub fn new_add_rospec(message_id: u32, rospec_id: u32) -> Self {
    let mut payload = BytesMut::new();

    let rospec = Self::encode_rospec(rospec_id);
    payload.extend_from_slice(&rospec);

    LlrpMessage::new(ADD_ROSPEC, message_id, payload.to_vec())
  }

  // NOTE: LLRP Parameters (4-byte header + all of the parameter fields) must be a multiple of 8 bytes.
  // Padding must be added if it is not.

  fn encode_rospec(rospec_id: u32) -> Vec<u8> {
    let mut buf = BytesMut::new();
    let start_pos = buf.len();

    // ROSpec Parameter Header
    buf.put_u16(PARAM_RO_SPEC);
    buf.put_u16(0); // Parameter length (Temp)

    // Fields
    buf.put_u32(rospec_id); // ROSpecID
    buf.put_u8(0);          // Priority
    buf.put_u8(0);          // CurrentState
    buf.put_u16(0);         // Reserved (Align to 32-bit boundary)

    // Encode ROBoundarySpec
    let ro_boundary_spec = Self::encode_ro_boundary_spec();
    buf.extend_from_slice(&ro_boundary_spec);

    // Encode AISpec
    let ai_spec = Self::encode_ai_spec();
    buf.extend_from_slice(&ai_spec);

    // Encode ROReportSpec
    let ro_report_spec = Self::encode_ro_report_spec();
    buf.extend_from_slice(&ro_report_spec);

    // Padding to align to 8 bytes
    let mut total_length = (buf.len() - start_pos) as u16;
    let required_padding = (8 - (total_length % 8)) % 8;

    for _ in 0..required_padding {
      buf.put_u8(0);
    }

    total_length = (buf.len() - start_pos) as u16;
    buf[start_pos + 2..start_pos + 4].copy_from_slice(&total_length.to_be_bytes());

    buf.to_vec()
  }

  fn encode_ro_boundary_spec() -> Vec<u8> {
    let mut buf = BytesMut::new();
    let start_pos = buf.len();

    // ROBoundarySpec Paramater Header
    buf.put_u16(PARAM_RO_BOUNDARY_SPEC);
    buf.put_u16(0); // Parameter Length (Temp)
  
    // ROSpecStartTrigger
    let start_trigger = Self::encode_ro_spec_start_trigger();
    buf.extend_from_slice(&start_trigger);

    // ROSpecStopTrigger
    let stop_trigger = Self::encode_ro_spec_stop_trigger();
    buf.extend_from_slice(&stop_trigger);

    // Padding to align to 8 bytes
    let total_length = (buf.len() - start_pos) as u16;
    buf[start_pos + 2..start_pos + 4].copy_from_slice(&total_length.to_be_bytes());

    buf.to_vec()
  }

  fn encode_ro_spec_start_trigger() -> Vec<u8> {
    let mut buf = BytesMut::new();
    let start_pos = buf.len();

    // ROSpecStartTrigger Parameter Header
    buf.put_u16(PARAM_RO_SPEC_START_TRIGGER);
    buf.put_u16(0); // Parameter length (Temp)

    // Fields
    buf.put_u8(START_TRIGGER_IMMEDIATE);  // ROSpecStartTriggerType
    buf.put_u8(0);                        // Reserved
    //buf.put_u32(0);                     // No further data

    // Padding to align to 8 bytes
    buf.put_slice(&[0u8; 6]);

    let total_length = (buf.len() - start_pos) as u16;
    buf[start_pos + 2..start_pos + 4].copy_from_slice(&total_length.to_be_bytes());

    buf.to_vec()
  }

  fn encode_ro_spec_stop_trigger() -> Vec<u8> {
    let mut buf = BytesMut::new();
    let start_pos = buf.len();
    
    // ROSpecStopTrigger Parameter Header
    buf.put_u16(PARAM_RO_SPEC_STOP_TRIGGER);
    buf.put_u16(0); // Parameter length (Temp)

    // Fields
    buf.put_u8(STOP_TRIGGER_DURATION);  // ROSpecStopTriggerType (0 = Null)
    buf.put_u8(0);                      // Reserved
    buf.put_u32(10_000);                // Duration (In milliseconds)

    // Padding to align to 8 bytes
    buf.put_u16(0);

    let total_length = (buf.len() - start_pos) as u16;
    buf[start_pos + 2..start_pos + 4].copy_from_slice(&total_length.to_be_bytes());

    buf.to_vec()
  }

  fn encode_ai_spec() -> Vec<u8> {
    let mut buf = BytesMut::new();
    let start_pos = buf.len();

    // AISpec Parameter Header
    buf.put_u16(PARAM_AI_SPEC);
    buf.put_u16(0); // Parameter length (Temp)

    // Antenna IDs
    buf.put_u16(PARAM_ANTENNA_ID); // Parameter Type for AntennaIDs
    buf.put_u16(0);                // Parameter Length (Temp)

    // Fields
    buf.put_u16(1);                // AntennaCount 
    buf.put_u16(0);                // AntennaID (0 = All Antennas)

    // AISpecStopTrigger
    let ai_spec_stop_trigger = Self::encode_ai_spec_stop_trigger();
    buf.extend_from_slice(&ai_spec_stop_trigger);

    // InventoryParameterSpec
    let inventory_param_spec = Self::encode_inventory_parameter_spec();
    buf.extend_from_slice(&inventory_param_spec);

    // Padding to align to 8 bytes
    let mut total_length = (buf.len() - start_pos) as u16;
    let required_padding = (8 - (total_length % 8)) % 8;

    for _ in 0..required_padding {
      buf.put_u8(0);
    }

    total_length = (buf.len() - start_pos) as u16;
    buf[start_pos + 2..start_pos + 4].copy_from_slice(&total_length.to_be_bytes());
    

    buf.to_vec()
  }

  fn encode_ai_spec_stop_trigger() -> Vec<u8> {
    let mut buf = BytesMut::new();
    let start_pos = buf.len();

    // AISpecStopTrigger Parameter Header
    buf.put_u16(PARAM_AI_SPEC_STOP_TRIGGER);
    buf.put_u16(0); // Parameter Length (Temp)

    // Fields
    buf.put_u8(AI_STOP_TRIGGER_DURATION); // AISpecStopTriggerType
    buf.put_u8(0);                        // Reserved
    buf.put_u32(5_000);                   // Duration in milliseconds

    // Padding to align to 8 bytes
    buf.put_u16(0);

    let total_length = (buf.len() - start_pos) as u16;
    buf[start_pos + 2..start_pos + 4].copy_from_slice(&total_length.to_be_bytes());

    buf.to_vec()
  }

  fn encode_inventory_parameter_spec() -> Vec<u8> {
    let mut buf = BytesMut::new();
    let start_pos = buf.len();
    
    // InventoryParameterSpec Parameter Header
    buf.put_u16(PARAM_INVENTORY_PARAMETER_SPEC);
    buf.put_u16(0); // Parameter Length (Temp)

    // Fields
    buf.put_u16(1); // InventoryParameterSpecID
    buf.put_u16(1); // ProtocolID (1 = EPCGlobalClass1Gen2)

    let total_length = (buf.len() - start_pos) as u16;
    buf[start_pos + 2..start_pos + 4].copy_from_slice(&total_length.to_be_bytes());

    buf.to_vec()
  }

  fn encode_ro_report_spec() -> Vec<u8> {
    let mut buf = BytesMut::new();
    let start_pos = buf.len();

    // ROReportSpec Parameter Header
    buf.put_u16(PARAM_RO_REPORT_SPEC);
    buf.put_u16(0); // Parameter length (Temp)

    // Fields
    buf.put_u8(0);  // ROReportTrigger (0 = None)
    buf.put_u16(1); // N (Number of tags before triggering a report)
    buf.put_u8(0);  // Reserved

    // Padding to align to 8 bytes
    buf.put_u32(0);

    // TagReportContentSelector
    let tag_report_content_selector = Self::encode_tag_report_content_selector();
    buf.extend_from_slice(&tag_report_content_selector);

    let total_length = (buf.len() - start_pos) as u16;
    buf[start_pos + 2..start_pos + 4].copy_from_slice(&total_length.to_be_bytes());

    buf.to_vec()
  }

  fn encode_tag_report_content_selector() -> Vec<u8> {
    let mut buf = BytesMut::new();
    let start_pos = buf.len();

    // TagReportContentSelect Parameter Header
    buf.put_u16(PARAM_TAG_REPORT_CONTENT_SELECTOR);
    buf.put_u16(0); // Parameter Length (Temp)

    // Fields (Content to enable in report)
    buf.put_u8(1); // EnableROSpecID
    buf.put_u8(1); // EnableSpecIndex
    buf.put_u8(1); // EnableInventoryParameterSpecID
    buf.put_u8(1); // EnableAntennaID
    buf.put_u8(1); // EnableChannelIndex
    buf.put_u8(1); // EnablePeakRSSI
    buf.put_u8(1); // EnableFirstSeenTimestamp
    buf.put_u8(1); // EnableLastSeenTimestamp
    buf.put_u8(1); // EnableTagSeenCount
    buf.put_u8(1); // EnableAccessSpecID

    // Padding to align to 8 bytes
    buf.put_u16(0);

    let total_length = (buf.len() - start_pos) as u16;
    buf[start_pos + 2..start_pos + 4].copy_from_slice(&total_length.to_be_bytes());

    buf.to_vec()
  }

  pub fn new_start_rospec(message_id: u32, rospec_id: u32) -> Self {
    let mut payload = BytesMut::new();

    // ROSpecID Parameter Header
    payload.put_u16(PARAM_ROSPEC_ID);
    payload.put_u16(6); // Parameter Length

    // ROSpecID Field
    payload.put_u32(rospec_id); // ROSpecID

    LlrpMessage::new(START_ROSPEC, message_id, payload.to_vec())
  }

  pub fn new_stop_rospec(message_id: u32, rospec_id: u32) -> Self {
    let mut payload = BytesMut::new();

    // ROSpecID Parameter Header
    payload.put_u16(PARAM_ROSPEC_ID);
    payload.put_u16(6); // Parameter Length

    // ROSpecID Field
    payload.put_u32(rospec_id); // ROSpecID

    LlrpMessage::new(STOP_ROSPEC, message_id, payload.to_vec())
  }

  pub fn encode(&self, protocol_version: u8) -> BytesMut {
    let mut buffer = BytesMut::with_capacity(self.message_length as usize);
    let version_and_reserved = (protocol_version as u16) << 10;

    buffer.put_u16(self.message_type);
    buffer.put_u16(version_and_reserved);
    buffer.put_u32(self.message_length);
    buffer.put_u32(self.message_id);
    buffer.extend_from_slice(&self.payload);
    
    buffer
  }

  pub fn decode(buf: &mut BytesMut) -> io::Result<Self> {
    if buf.len() < 10 {
      return Err(Error::new(ErrorKind::InvalidData, "Buffer too short for LLRP header"));
    }

    let message_type         = buf.get_u16();
    let version_and_reserved = buf.get_u16();
    let protocol_version = (version_and_reserved >> 10) & 0x3F;
    let message_length       = buf.get_u32();
    let message_id           = buf.get_u32();

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