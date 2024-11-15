use bytes::{BytesMut, Buf, BufMut};
use std::io::{self, Error, ErrorKind};

// Message type constants for LLRP operations.
// Used to define various types of LLRP messages and their responses.
pub const TYPE_GET_READER_CAPABILITIES          : u16 = 1;
pub const TYPE_GET_READER_CAPABILITIES_RESPONSE : u16 = 11;
pub const TYPE_GET_READER_CONFIG                : u16 = 2;
pub const TYPE_GET_READER_CONFIG_RESPONSE       : u16 = 12;
pub const TYPE_SET_READER_CONFIG                : u16 = 3;
pub const TYPE_SET_READER_CONFIG_RESPONSE       : u16 = 13;
pub const TYPE_CLOSE_CONNECTION                 : u16 = 14;
pub const TYPE_CLOSE_CONNECTION_RESPONSE        : u16 = 4;
pub const TYPE_ADD_ROSPEC                       : u16 = 20;
pub const TYPE_ADD_ROSPEC_RESPONSE              : u16 = 30;
pub const TYPE_DELETE_ROSPEC                    : u16 = 21;
pub const TYPE_DELETE_ROSPEC_RESPONSE           : u16 = 31;
pub const TYPE_START_ROSPEC                     : u16 = 22;
pub const TYPE_START_ROSPEC_RESPONSE            : u16 = 32;
pub const TYPE_STOP_ROSPEC                      : u16 = 23;
pub const TYPE_STOP_ROSPEC_RESPONSE             : u16 = 33;
pub const TYPE_ENABLE_ROSPEC                    : u16 = 24;
pub const TYPE_ENABLE_ROSPEC_RESPONSE           : u16 = 34;
pub const TYPE_DISABLE_ROSPEC                   : u16 = 25;
pub const TYPE_DISABLE_ROSPEC_RESPONSE          : u16 = 35;
pub const TYPE_GET_ROSPECS                      : u16 = 26;
pub const TYPE_GET_ROSPECS_RESPONSE             : u16 = 36;
pub const TYPE_GET_REPORT                       : u16 = 60;
pub const TYPE_RO_ACCESS_REPORT                 : u16 = 61;
pub const TYPE_KEEPALIVE                        : u16 = 62;
pub const TYPE_KEEPALIVE_ACK                    : u16 = 72;
pub const TYPE_READER_EVENT_NOTIFICATION        : u16 = 63;
pub const TYPE_ENABLE_EVENTS_AND_REPORTS        : u16 = 64;
pub const TYPE_ERROR_MESSAGE                    : u16 = 100;
pub const TYPE_CUSTOM_MESSAGE                   : u16 = 1023;

// Parameter type constants for LLRP parameters.
// Definss the types of parameters used in LLRP messages.
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

  pub message_type   : u16,
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
/// - `param_type`: The 16-bit type enumerator for the parameter.
/// - `payload`: A vector of nested `Parameter` instances.
#[derive(Debug)]
struct Parameter {
  /// Nested-parameter type
  param_type: u16, 

  // Parameter's value/payload.
  payload: Vec<Parameter>,
}

impl LlrpMessage {
  
  /// Constructs a new LLRP message with the specified type, ID, and payload.
  ///
  /// Automatically calculates the message length based on the payload size.
  pub fn new(message_type: u16, message_id: u32, payload: Vec<u8>) -> Self {
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
  pub fn new_enable_events_and_reports(message_id: u32) -> Self {
    LlrpMessage::new(TYPE_ENABLE_EVENTS_AND_REPORTS, message_id, vec![])
  }

  /// Constructs a new `AddROSpec` message with the specified ROSpec ID.
  ///
  /// The ROSpec includes the following parameters:
  /// - `ROBoundarySpec`: Specifies start and stop triggers.
  /// - `AISpec`: Defines antenna configurations and stop triggers.
  /// - `ROReportSpec`: Configures report generation.
  pub fn new_add_rospec(message_id: u32, rospec_id: u32) -> Self {
    
    let ro_boundary_spec = Parameter {
      param_type: PARAM_RO_BOUNDARY_SPEC,
      payload: vec![]
    };

    let ai_spec = Parameter {
      param_type: PARAM_AI_SPEC,
      payload: vec![]
    };

    let ro_report_spec = Parameter {
      param_type: PARAM_RO_REPORT_SPEC,
      payload: vec![]
    };

    let ro_spec = Parameter {
      param_type: PARAM_RO_SPEC,
      payload: vec![ro_boundary_spec, ai_spec, ro_report_spec]
    };

    let mut payload = BytesMut::new();

    // Recursive function to encode parameters into the payload.
    fn encode_parameter(param: &Parameter, buffer: &mut BytesMut, rospec_id: u32) {
      
      let initial_length_pos = buffer.len();
      buffer.put_u16(param.param_type);
      buffer.put_u16(0x0000); // Placeholder for length, updated later.

      // Encode specific fields based on the parameter type.
      match param.param_type {

        PARAM_RO_SPEC => {
          buffer.put_u32(rospec_id);
          buffer.put_u8(0x00); // Priority
          buffer.put_u8(0x00); // CurrentState
        }

        /* LLRP Standard - Release 2.0, 11.2.1.1

          ROBoundarySpecParameter

          ----------
          ROSpecStartTrigger: ROSpecStartTrigger Parameter
          ----------
          ROSpecStopTrigger: ROSpecStopTrigger Parameter
        */
        PARAM_RO_BOUNDARY_SPEC => {

          /* LLRP Standard - Release 2.0, 11.2.1.1.1

            ROSpecStartTriggerParameter

            ----------
            ROSpecStartTriggerType: Integer
            
            Possible Values:

            0 Null – No start trigger. The only way to start the ROSpec is with a START_ROSPEC from the Client.

            1 Immediate

            2 Periodic

            3 GPI
            ----------
            PeriodicTriggerValue: PeriodicTriggerValue Parameter [Optional]. This parameter SHALL be present when ROSpecStartTriggerType = 2.
            ----------
            GPITriggerValue: GPITriggerValue Parameter [Optional]. This parameter SHALL be present when ROSpecStartTriggerType = 3.
          */
          buffer.put_u16(PARAM_RO_SPEC_START_TRIGGER);
          buffer.put_u16(0x0005); // Length

          /* Fields */
          
          //ROSpecStartTriggerType
          buffer.put_u8(0x01); // 1 - Immediate

          /* LLRP Standard - Release 2.0, 11.2.1.1.4

            ROSpecStopTriggerParameter

            ----------
            ROSpecStopTriggerType: Integer

            Possible Values:

            0 Null – Stop when all Specs are done (including any looping as required by a LoopSpec parameter), or when
            preempted, or with a STOP_ROSPEC from the Client.

            1 Duration – Stop after DurationTriggerValue milliseconds, or when all Specs are done
            (including any looping as required by a LoopSpec parameter), or when preempted, or with a STOP_ROSPEC
            from the Client.

            2 GPI with a timeout value – Stop when a GPI "fires", or after Timeout milliseconds, or when all Specs are done
            (including any looping as required by a LoopSpec parameter), or when preempted, or with a STOP_ROSPEC
            from the Client.

            DurationTriggerValue: Duration in milliseconds. This field is ignored when ROSpecStopTriggerType != 1.

            GPITriggerValue: GPITriggerValue Parameter [Optional]. This parameter SHALL be present when ROSpecStopTriggerType = 2.
          */
          buffer.put_u16(PARAM_RO_SPEC_STOP_TRIGGER);
          buffer.put_u16(0x0009); // Length
          
          /* Fields */

          // ROSpecStopTriggerType
          buffer.put_u8(0x00); // 0 - No stop trigger

          buffer.put_u32(0x00000000); // Null-field padding (Fields not required with ROSpecStoTriggerType=0)
        }

        /* LLRP Standard - Release 2.0, 11.2.2

          AISpec Parameter

          ----------
          AISpecStopTrigger: <AISpecStopTrigger Parameter>
          ----------
          AntennaIDs: Short Array. If this set contains an antenna ID of zero, this AISpec will utilise all the antennas of the Reader.
          ----------
          InventoryParameterSpecs: <List of InventoryParameterSpec Parameter>
          ----------
          Custom Extension Point List: List of <custom Parameter> [Optional]
        */
        PARAM_AI_SPEC => {

          let antenna_ids = vec![0x0000]; // 0 - Use all antennas

          // AntennaID Array (Allocated before AISpecStopTrigger)
          for antenna_id in antenna_ids {
            buffer.put_u16(antenna_id);
          }

          /* LLRP Standard - Release 2.0, 11.2.2.1

            AISpecStopTriggerParameter

            ----------
            AISpecStopTriggerType: Integer

            Possible Values:
            0 Null – Stop when ROSpec is done.

            1 Duration

            2 GPI with a timeout value

            3 Tag observation
            ----------
            Duration Trigger: Unsigned Integer. Duration of AISpec in milliseconds. This field SHALL be ignored when AISpecStopTriggerType != 1.
            ----------
            GPI Trigger : GPITrigger value Parameter [Optional]. This field SHALL be present when AISpecStopTriggerType = 2.
            ----------
            TagObservation Trigger : TagObservation Trigger Parameter [Optional]. This field SHALL be present when AISpecStopTriggerType = 3.
          */
          buffer.put_u16(PARAM_AI_SPEC_STOP_TRIGGER);
          buffer.put_u16(0x0009);

          /* Fields */

          // AISpecStopTriggerType
          buffer.put_u8(0x0); // 0 - Stop when ROSpec is done

          buffer.put_u32(0x00000000); // Null-field padding (Fields not required with AISpecStopTriggerType=0)
        }

        /* LLRP Standard - Release 2.0, 14.2.1

          ROReportSpecParameter

          ----------
          ROReportTrigger: Integer

          Possible Values:

          0 None

          1 (Upon N TagReportData Parameters or End of AISpec) Or (End of RFSurveySpec) - N=0 is unlimited

          2 Upon N TagReportData Parameters or End of ROSpec - N=0 is unlimited

          3 Upon N seconds or (End of AISpec or End of RFSurveySpec) – N=0 is unlimited

          4 Upon N seconds or End of ROSpec – N=0 is unlimited.

          5 Upon N milliseconds or (End of AISpec or End of RFSurveySpec) – N=0 is unlimited

          6 Upon N milliseconds or End of ROSpec – N=0 is unlimited
          
          7 Upon N inventory rounds or End of ROSpec - N=0 is unlimited. (If N=1, the TagSeenCount parameter in
          TagReportData can be used as well
          ----------
          N: Unsigned Short Integer. When ROReportTrigger = 1 or 2, this is the number of TagReportData parameters 
          present in a report before the report trigger fires. When ROReportTrigger = 3 or 4, this is the number 
          of seconds since the last report was generated before the report trigger fires. When ROReportTrigger = 5 or 6, this is the number of
          milliseconds since the last report was generated before the report trigger fires. If N = 0, there is no limit on either the number of 
          TagReportData parameters, or the time since the last report was generated. This field SHALL be ignored when ROReportTrigger = 0.
          ----------
          ReportContents: <TagReportContentSelector Parameter>
          ----------
          Custom Extension Point List: List of <Custom Parameter> [Optional]
        */
        PARAM_RO_REPORT_SPEC => {
          // ROReportTriggerType
          buffer.put_u8(0x00); // 0 - None
          buffer.put_u16(0x0000); // N null-field padding (Fields not required with ROReportTriggerType=0)

          /* LLRP Standard - Release 2.0, 14.2.1.1

            TagReportContentSelector

            Note: All booleans are encoded as single bits within an unsigned 16-bit word

            ----------
            EnableROSpecID: Boolean EnableSpecIndex:
            ----------
            Boolean EnableInventoryParameterSpecID:
            ----------
            Boolean EnableAntennaID: Boolean
            ----------
            EnableChannelIndex: Boolean
            ----------
            EnablePeakRSSI: Boolean
            ----------
            EnableFirstSeenTimestamp: Boolean
            ----------
            EnableLastSeenTimestamp: Boolean
            ----------
            EnableTagSeenCount: Boolean
            ----------
            EnableCryptoResponse: Boolean

          */
          buffer.put_u16(PARAM_TAG_REPORT_CONTENT_SELECTOR);
          buffer.put_u16(0x0006);

          /* Fields */

          // ReportContentSelector boolean bit values
          buffer.put_u16(0x0000); // ReportContentSelector (TagInfo/EPC)
        }
        _ => {}
      }

      // Recursively encode nested parameters.
      for sub_param in &param.payload {
        encode_parameter(sub_param, buffer, rospec_id); // Recursive call
      }

      let final_length_pos = buffer.len();
      let actual_length = (final_length_pos - initial_length_pos) as u16;

      buffer[initial_length_pos + 2..initial_length_pos + 4].copy_from_slice(&actual_length.to_be_bytes());
    };

    encode_parameter(&ro_spec, &mut payload, rospec_id);

    LlrpMessage::new(TYPE_ADD_ROSPEC, message_id, payload.to_vec())
  }

  pub fn new_start_rospec(message_id: u32, rospec_id: u32) -> Self {
    let mut payload = BytesMut::with_capacity(4);
    payload.put_u32(rospec_id);
    LlrpMessage::new(TYPE_START_ROSPEC, message_id, payload.to_vec())
  }

  pub fn new_stop_rospec(message_id: u32, rospec_id: u32) -> Self {
    let mut payload = BytesMut::with_capacity(4);
    payload.put_u32(rospec_id);
    LlrpMessage::new(TYPE_STOP_ROSPEC, message_id,   payload.to_vec())
  }

  pub fn new_delete_rospec(message_id: u32, rospec_id: u32) -> Self {
    let mut payload = BytesMut::with_capacity(4);
    payload.put_u32(rospec_id);
    LlrpMessage::new(TYPE_DELETE_ROSPEC, message_id, payload.to_vec())
  }

  pub fn new_close_connection(message_id: u32) -> Self {
    LlrpMessage::new(TYPE_CLOSE_CONNECTION , message_id, vec![])
  }

  /// Encodes the LLRP message into a binary format.
  ///
  /// This includes the LLRP header and the message payload.
  pub fn encode(&self) -> BytesMut {
    let mut buffer = BytesMut::with_capacity(self.message_length as usize);

    let version = 0x0001;
    let reserved = 0x0000;
    let version_and_type = ((version & 0x7) << 13) | ((reserved & 0x7) << 10) | (self.message_type & 0x3FFF);

    buffer.put_u16(version_and_type as u16);
    buffer.put_u32(self.message_length);
    buffer.put_u32(self.message_id);
    buffer.extend_from_slice(&self.payload);

    buffer
  }

  /// Decodes an LLRP message from a binary buffer.
  ///
  /// Returns an `io::Result` with the decoded message or an error.
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

fn calculate_total_length(param: &Parameter) -> u16 {
  let mut total_length = 4;

  for sub_param in &param.payload {
      total_length += calculate_total_length(sub_param);
  }

  total_length += param.payload.len() as u16;

  total_length
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