use std::{fmt, io::{self, Error, ErrorKind}};
use bytes::{Buf, BytesMut};
use log::{debug, warn};

use crate::llrp::{LlrpParameter, LlrpParameterType};

#[derive(Debug)]
pub enum LlrpParameterData {
  LLRPStatus                (LLRPStatus),
  GeneralDeviceCapabilities (GeneralDeviceCapabilities),
  LLRPCapabilities          (LLRPCapabilities),
  RegulatoryCapabilities    (RegulatoryCapabilities),
  C1G2LLRPCapabilities      (C1G2LLRPCapabilities),
  Identification            (Identification)
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

#[derive(Debug)]
pub struct LLRPStatus {
  pub status_code : u16,
  pub error_desc  : u16
}

impl LLRPStatus {
  pub fn decode(
    buf: &[u8]
  ) -> io::Result<Self> {

    let mut buf = BytesMut::from(buf);

    if buf.remaining() < 4 {
      return Err(Error::new(
        ErrorKind::InvalidData,
        "Buffer too short for LLRPStatus"
      ));
    }

    let status_code = buf.get_u16();
    let error_desc = buf.get_u16();

    Ok(LLRPStatus { 
      status_code, 
      error_desc
    })
  }
}

#[derive(Debug)]
pub struct GeneralDeviceCapabilities {
  pub max_number_of_antennas_supported  : u16,
  pub general_device_capabilities       : u16,
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

    if buf.remaining() < 12 {
      return Err(Error::new(
        ErrorKind::InvalidData,
        "Buffer too short for GeneralDeviceCapabilities"
      ));
    }

    let max_number_of_antennas_supported = buf.get_u16();
    let general_device_capabilities = buf.get_u16();
    let device_manufacturer_name = buf.get_u32();
    let model_name = buf.get_u32();

    if buf.remaining() < 2 {
      return Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "Buffer too short for firmware length prefix",
      ));
    }

    let firmware_length = buf.get_u16() as usize;

    if buf.remaining() < firmware_length {
      return Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "Buffer too short for firmware version string",
      ));
    }

    let firmware_bytes = buf.split_to(firmware_length);
    let reader_firmware_version = String::from_utf8(firmware_bytes.to_vec())
      .map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?;

    let sub_param_slice = buf.chunk();
    let sub_parameters = parse_parameters(sub_param_slice)?;

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
      general_device_capabilities,
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
  pub client_request_op_spec_timeout                : u16,
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
    let client_request_op_spec_timeout                 = buf.get_u16();
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
  pub communications_standard : u16,
  pub uhf_band_capabilities   : Option<UHFBandCapabilities>
}

impl RegulatoryCapabilities {
  pub fn decode(
    buf: &[u8]
  ) -> io::Result<Self> {

    let mut buf = BytesMut::from(buf);

    if buf.remaining() < 4 {
      return Err(Error::new(
        ErrorKind::InvalidData,
        "Buffer too short for RegulatoryCapabilities"
      ));
    }

    let country_code = buf.get_u16();
    let communications_standard = buf.get_u16();

    let param_slice = buf.chunk();
    let sub_parameters = parse_parameters(param_slice)?;

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
  pub transmit_power_levels  : Vec<TransmitPowerLevelTableEntry>,
  pub frequency_information  : Option<FrequencyInformation>,
  pub c1g2_uhf_rf_mode_table : Option<C1G2UHFRFModeTable>
}

impl UHFBandCapabilities {
  pub fn decode(
    buf: &[u8]
  ) -> io::Result<Self> {
    let mut buf = BytesMut::from(buf);
    let sub_parameters = parse_parameters(&mut buf)?;

    let mut transmit_power_levels = Vec::new();
    let mut frequency_information = None;
    let mut c1g2_uhf_rf_mode_table = None;

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

        LlrpParameterType::C1G2UHFRFModeTable => {
          let c1g2_table = C1G2UHFRFModeTable::decode(&param.param_value)?;
          c1g2_uhf_rf_mode_table = Some(c1g2_table);
        }

        _ => {
          warn!("Unhandled sub-parameter type in UHFBandCapabilities: {:?}", param.param_type);
        }
      }
    }

    Ok(UHFBandCapabilities {
      transmit_power_levels,
      frequency_information,
      c1g2_uhf_rf_mode_table
    })
  }
}

#[derive(Debug)]
pub struct TransmitPowerLevelTableEntry {
  pub index                : u16,
  pub transmit_power_value : u16
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
  pub index                     : u16,
  pub receive_sensitivity_value : i16
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
  pub hop_table_id   : u16,
  pub number_of_hops : u16,
  pub frequencies    : Vec<u32>
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
    let number_of_hops = buf.get_u16();
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
      number_of_hops,
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
pub struct C1G2UHFRFModeTable {
  pub entries: Vec<C1G2UHFRFModeTableEntry>
}

impl C1G2UHFRFModeTable {
  pub fn decode(
    buf: &[u8]
  ) -> io::Result<Self> {

    let mut buf = BytesMut::from(buf);
    let sub_parameters = parse_parameters(&buf)?;

    let mut entries = Vec::new();

    for param in sub_parameters {
      if param.param_type == LlrpParameterType::C1G2UHFRFModeTableEntry {
        let entry = C1G2UHFRFModeTableEntry::decode(&param.param_value)?;
        entries.push(entry);
      } else {
        warn!("Unexpected parameter type in C1G2UHFRFModeTable: {:?}", param.param_type);
      }
    }

    Ok(C1G2UHFRFModeTable { entries })
  }
}

#[derive(Debug)]
pub struct C1G2UHFRFModeTableEntry {
  pub mode_identifier             : u32,
  pub dr                          : bool,
  pub epc_hag_t_and_c_conformance : bool,
  pub m                           : u8,
  pub forward_link_modulation     : u8,
  pub spectral_mask_indicator     : u8,
  pub bdr                         : u32,
  pub pie                         : u32,
  pub min_tari                    : u32,
  pub max_tari                    : u32,
  pub tari_step                   : u32  
}

impl C1G2UHFRFModeTableEntry {
  pub fn decode(
    buf: &[u8]
  ) -> io::Result<Self> {

    let mut buf = BytesMut::from(buf);

    if buf.remaining() < 2 {
      return Err(Error::new(
        ErrorKind::InvalidData,
        "Buffer too short for C1G2UHFRFModeTableEntry header"
      ));
    }

    let mode_identifier = buf.get_u32();
    
    let flags = buf.get_u8();
    let dr = (flags & 0x80) != 0;
    let epc_hag_t_and_c_conformance = (flags & 0x40) != 0;

    let m = buf.get_u8();
    let forward_link_modulation = buf.get_u8();
    let spectral_mask_indicator = buf.get_u8();
    let bdr = buf.get_u32();
    let pie = buf.get_u32();
    let min_tari = buf.get_u32();
    let max_tari = buf.get_u32();
    let tari_step = 0;

    Ok(C1G2UHFRFModeTableEntry {
      mode_identifier,
      dr,
      epc_hag_t_and_c_conformance,
      m,
      forward_link_modulation,
      spectral_mask_indicator,
      bdr,
      pie,
      min_tari,
      max_tari,
      tari_step
    })
  }
}

#[derive(Debug)]
pub struct C1G2LLRPCapabilities {
  pub supports_block_erase                : bool,
  pub supports_block_write                : bool,
  pub supports_block_permalock            : bool,
  pub supports_tag_recommissioning        : bool,
  pub supports_umi_method_2               : bool,
  pub supports_xpc                        : bool,
  pub max_number_select_filters_per_query : u16
}

impl C1G2LLRPCapabilities {
  pub fn decode(
    buf: &[u8]
  ) -> io::Result<Self> {

    let mut buf = BytesMut::from(buf);

    if buf.remaining() < 1 {
      return Err(Error::new(
        ErrorKind::InvalidData,
        "Buffer too short for C1G2LLRPCapabilities"
      ));
    }

    let flags = buf.get_u8();
    let supports_block_erase         = (flags & 0x80) != 0;
    let supports_block_write         = (flags & 0x40) != 0;
    let supports_block_permalock     = (flags & 0x20) != 0;
    let supports_tag_recommissioning = (flags & 0x10) != 0;
    let supports_umi_method_2        = (flags & 0x08) != 0;
    let supports_xpc                 = (flags & 0x04) != 0;
    
    let max_number_select_filters_per_query = buf.get_u16();

    Ok(C1G2LLRPCapabilities {
      supports_block_erase,
      supports_block_write,
      supports_block_permalock,
      supports_tag_recommissioning,
      supports_umi_method_2,
      supports_xpc,
      max_number_select_filters_per_query
    })
  }
}

#[derive(Debug)]
pub struct Identification {
  pub id_type   : u8,
  pub reader_id : Vec<u8>
}

impl Identification {
  pub fn decode(
    buf: &[u8]
  ) -> io::Result<Self> {
    
    if buf.len() < 1 {
      return Err(Error::new(
        ErrorKind::InvalidData,
        "Buffer too short for Identification parameter, missing IDType"
      ));
    }

    let length = buf.len();
    debug!("Length: {:?}", length);

    let id_type = buf[0];
    let reader_id = buf[1..].to_vec();

    match id_type {
      
      0 => {

        if reader_id.len() < 8 {
          return Err(Error::new(
            ErrorKind::InvalidData,
            format!(
              "Identification parameter: Expected 8-byte MAX address, received {} bytes",
              reader_id.len()
            )
          ));
        };

        if reader_id.len() > 8 {
          warn!("Identification parameter: Extra bytes detected for MAC address: {}", reader_id.len() - 8);
        }
      }

      1 => {
        // IDType = 1: EPC is variable-length, no additional checks required.
        if reader_id.is_empty() {
          warn!("Identification parameter: EPC (IDType=1) is empty");
        }
      }

      _ => {
        warn!("Unknown IDType in Identification parameter: {}", id_type);
      }
    }

    let decoded_length = 1 + reader_id.len();
    if decoded_length != decoded_length {
      warn!(
        "Identification parameter: Expected length ({}) does not match decoded length ({})",
        length, decoded_length
    )}
    
    Ok(Identification {
      id_type,
      reader_id
    })
  }
}

pub fn parse_parameters(buf: &[u8]) -> io::Result<Vec<LlrpParameter>> {

  let mut parameters = Vec::new();
  let mut index = 0;
  let buf_len = buf.len();

  while index < buf_len {

    if buf_len - index < 1 {
      return Err(Error::new(
        ErrorKind::InvalidData,
        "Insufficient data for parameter parsing",
      ));
    }

    let first_byte = buf[index];
    if (first_byte & 0x80) != 0 {

      let param_type_value = first_byte & 0x7F;
      index += 1;

      let param_type = LlrpParameterType::from_value(param_type_value as u16);
      let param_value_length = get_tv_param_length(param_type.unwrap_or(LlrpParameterType::Custom));
      
      if let Some(param_value_length) = param_value_length {

        if buf_len - index < param_value_length {
          return Err(Error::new(
            ErrorKind::InvalidData,
            "Buffer too short for TV parameter value",
          ));
        }

        let param_value = buf[index..index + param_value_length].to_vec();
        index += param_value_length;

        let parameter = LlrpParameter {
          param_type: param_type.unwrap_or(LlrpParameterType::Custom),
          param_length: (1 + param_value_length) as u16,
          param_value,
          sub_params: None,
        };

        parameters.push(parameter);

      } else {
        return Err(Error::new(
          ErrorKind::InvalidData,
          format!("Unknown TV parameter length for parameter type {:?}", param_type),
        ));
      }

    } else {

      if buf_len - index < 4 {
        return Err(Error::new(
          ErrorKind::InvalidData,
          "Buffer too short for TLV parameter header",
        ));
      }

      let param_type_value = ((buf[index] as u16) << 8) | buf[index + 1] as u16;
      index += 2;

      let param_length = ((buf[index] as u16) << 8) | buf[index + 1] as u16;
      index += 2;

      if param_length < 4 || (param_length - 4) as usize > (buf_len - index) {
        return Err(Error::new(
          ErrorKind::InvalidData,
          "Invalid TLV parameter length",
        ));
      }

      let param_value_length = (param_length - 4) as usize;
      let param_value = buf[index..index + param_value_length].to_vec();
      index += param_value_length;

      let param_type = LlrpParameterType::from_value(param_type_value);
      let parameter = LlrpParameter {
        param_type: param_type.unwrap_or(LlrpParameterType::Custom),
        param_length,
        param_value,
        sub_params: None,
      };

      parameters.push(parameter);
    }
  }

  Ok(parameters)
}

pub fn get_tv_param_length(param_type: LlrpParameterType) -> Option<usize> {
  match param_type {
    LlrpParameterType::EPC96 => Some(12),
    _ => None
  }
}