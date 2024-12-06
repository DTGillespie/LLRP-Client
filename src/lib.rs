use std::os::raw::c_char;
use std::ffi::{CStr, CString};
use std::ptr;
use std::sync::Mutex;
use llrp::LlrpResponseData;
use tokio::runtime::Runtime;
use lazy_static::lazy_static;

mod client;
mod config;
mod llrp;
mod params;

use client::LlrpClient;

type ReaderCapabilitiesCallback = extern "C" fn(capabilities: *const c_char);
type ReaderConfigCallback       = extern "C" fn(config: *const c_char);
type ROAccessReportCallback     = extern "C" fn(report: *const c_char);

lazy_static! {
  static ref RUNTIME: Runtime = Runtime::new().unwrap();
  static ref LAST_ERROR                   : Mutex<Option<String>>                     = Mutex::new(None);
  static ref READER_CAPABILITIES_CALLBACK : Mutex<Option<ReaderCapabilitiesCallback>> = Mutex::new(None);
  static ref READER_CONFIG_CALLBACK       : Mutex<Option<ReaderConfigCallback>>       = Mutex::new(None);
  static ref RO_ACCESS_REPORT_CALLBACK    : Mutex<Option<ROAccessReportCallback>>     = Mutex::new(None);
}

#[no_mangle]
pub extern "C" fn set_reader_capabilities_callback(callback: ReaderCapabilitiesCallback) {
  *READER_CAPABILITIES_CALLBACK.lock().unwrap() = Some(callback);
}

#[no_mangle]
pub extern "C" fn set_reader_config_callback(callback: ReaderConfigCallback) {
  *READER_CONFIG_CALLBACK.lock().unwrap() = Some(callback);
}

#[no_mangle]
pub extern "C" fn set_ro_access_report_callback(callback: ROAccessReportCallback) {
  *RO_ACCESS_REPORT_CALLBACK.lock().unwrap() = Some(callback);
}

pub struct LlrpClientWrapper(LlrpClient);

#[no_mangle]
pub extern "C" fn initialize_client(config_path: *const c_char) -> *mut LlrpClientWrapper {

  let config_path: String = unsafe {
    
    if config_path.is_null() {
      set_last_error("Null config path pointer");
      return ptr::null_mut();
    }

    CStr::from_ptr(config_path).to_string_lossy().into_owned()
  };

  let client_result = RUNTIME.block_on(LlrpClient::initialize(config_path.as_str()));

  match client_result {
    Ok(client) => Box::into_raw(Box::new(LlrpClientWrapper(client))),
    Err(e) => {
      set_last_error(&e.to_string());
      ptr::null_mut()
    }
  }
}

#[no_mangle]
pub extern "C" fn send_keep_alive(client_ptr: *mut LlrpClientWrapper) -> i32 {
  unsafe {

    if client_ptr.is_null() {
      set_last_error("Null client pointer");
      return -1;
    }

    let client = &mut *client_ptr;

    match RUNTIME.block_on(client.0.send_keep_alive()) {
      Ok(_) => 0,  
      Err(e) => {
        set_last_error(&e.to_string());
        -1
      }
    }
  }
}

#[no_mangle]
pub extern "C" fn send_enable_events_and_reports(client_ptr: *mut LlrpClientWrapper) -> i32 {
  unsafe {

    if client_ptr.is_null() {
      set_last_error("Null client pointer");
      return -1;
    }

    let client = &mut *client_ptr;

    match RUNTIME.block_on(client.0.send_enable_events_and_reports()) {
      Ok(_) => 0,  
      Err(e) => {
        set_last_error(&e.to_string());
        -1
      }
    }
  }
}

#[no_mangle]
pub extern "C" fn send_get_reader_capabilities(client_ptr: *mut LlrpClientWrapper) -> i32 {
  unsafe {

    if client_ptr.is_null() {
      set_last_error("Null client pointer");
      return -1;
    }

    let client = &mut *client_ptr;
    let callback_lock = READER_CAPABILITIES_CALLBACK.lock().unwrap();

    if callback_lock.is_none() {
      set_last_error("No ReaderCapabilities callback registered");
      return -1;
    }

    let callback = callback_lock.unwrap();

    match RUNTIME.block_on(client.0.send_get_reader_capabilities(move | response_data | async move {

      let capabilities_str = match response_data {

        LlrpResponseData::ReaderCapabilities(parameters) => {
          format!("{:?}", parameters)
        }

        _ => "Unexpected GetReaderCapabilities response".to_string()

      };

      let c_capabilities = CString::new(capabilities_str).unwrap();
      callback(c_capabilities.as_ptr());

    })) {
      Ok(_) => 0,  
      Err(e) => {
        set_last_error(&e.to_string());
        -1
      }
    }
  }
}

#[no_mangle]
pub extern "C" fn send_get_reader_config(client_ptr: *mut LlrpClientWrapper) -> i32 {
  unsafe {

    if client_ptr.is_null() {
      set_last_error("Null client pointer");
      return -1;
    }

    let client = &mut *client_ptr;
    let callback_lock = READER_CONFIG_CALLBACK.lock().unwrap();

    if callback_lock.is_none() {
      set_last_error("No ReaderConfig callback registered");
      return -1;
    }

    let callback = callback_lock.unwrap();

    match RUNTIME.block_on(client.0.send_get_reader_config(move | response_data | async move {

      let config_str = match response_data {

        LlrpResponseData::ReaderConfig(parameters) => {
          format!("{:?}", parameters)
        }

        _ => "Unexpected GetReaderConfig response".to_string()
      };

      let c_config = CString::new(config_str).unwrap();
      callback(c_config.as_ptr());

    })) {
      Ok(_) => 0,
      Err(e) => {
        set_last_error(&e.to_string());
        -1
      }
    }
  }
}

#[no_mangle]
pub extern "C" fn send_set_reader_config(client_ptr: *mut LlrpClientWrapper) -> i32 {
  unsafe {

    if client_ptr.is_null() {
      set_last_error("Null client pointer");
      return -1;
    }

    let client = &mut *client_ptr;

    match RUNTIME.block_on(client.0.send_set_reader_config()) {
      Ok(_) => 0,
      Err(e) => {
        set_last_error(&e.to_string());
        -1
      }
    }
  }
}

#[no_mangle]
pub extern "C" fn send_add_rospec(client_ptr: *mut LlrpClientWrapper) -> i32 {
  unsafe {

    if client_ptr.is_null() {
      set_last_error("Null client pointer");
      return -1;
    }

    let client = &mut *client_ptr;

    match RUNTIME.block_on(client.0.send_add_rospec()) {
      Ok(_) => 0,
      Err(e) => {
        set_last_error(&e.to_string());
        -1
      }
    }
  }
}

#[no_mangle]
pub extern "C" fn send_enable_rospec(client_ptr: *mut LlrpClientWrapper) -> i32 {
  unsafe {

    if client_ptr.is_null() {
      set_last_error("Null client pointer");
      return -1;
    }

    let client = &mut *client_ptr;

    match RUNTIME.block_on(client.0.send_enable_rospec()) {
      Ok(_) => 0,
      Err(e) => {
        set_last_error(&e.to_string());
        -1
      }
    }
  }
}

#[no_mangle]
pub extern "C" fn send_start_rospec(client_ptr: *mut LlrpClientWrapper) -> i32 {
  unsafe {

    if client_ptr.is_null() {
      set_last_error("Null client pointer");
      return -1;
    }

    let client = &mut *client_ptr;

    match RUNTIME.block_on(client.0.send_start_rospec()) {
      Ok(_) => 0,
      Err(e) => {
        set_last_error(&e.to_string());
        -1
      }
    }
  }
}

#[no_mangle]
pub extern "C" fn send_stop_rospec(client_ptr: *mut LlrpClientWrapper) -> i32 {
  unsafe {

    if client_ptr.is_null() {
      set_last_error("Null client pointer");
      return -1;
    }

    let client = &mut *client_ptr;

    match RUNTIME.block_on(client.0.send_stop_rospec()) {
      Ok(_) => 0,
      Err(e) => {
        set_last_error(&e.to_string());
        -1
      }
    }
  }
}

#[no_mangle]
pub extern "C" fn send_delete_rospec(client_ptr: *mut LlrpClientWrapper, rospec_id: u32) -> i32 {
  unsafe {

    if client_ptr.is_null() {
      set_last_error("Null client pointer");
      return -1;
    }

    let client = &mut *client_ptr;

    match RUNTIME.block_on(client.0.send_delete_rospec(rospec_id)) {
      Ok(_) => 0,
      Err(e) => {
        set_last_error(&e.to_string());
        -1
      }
    }
  }
}

#[no_mangle]
pub extern "C" fn await_ro_access_report(client_ptr: *mut LlrpClientWrapper) -> i32 {
  unsafe {

    if client_ptr.is_null() {
      set_last_error("Null client pointer");
      return -1;
    }

    let client = &mut *client_ptr;
    let callback_lock = RO_ACCESS_REPORT_CALLBACK.lock().unwrap();

    if callback_lock.is_none() {
      set_last_error("No ROAccessReport callback registered");
      return -1;
    }

    let callback = callback_lock.unwrap();

    match RUNTIME.block_on(client.0.await_ro_access_report(move | response_data | async move {

      let report_str = match response_data {
        
        LlrpResponseData::TagReport(epc_data) => {
          format!("{:?}", epc_data)
        }

        _ => "Unexpected ROAccessReport response".to_string()
      };

      let c_report = CString::new(report_str).unwrap();
      callback(c_report.as_ptr());

    })) {
      Ok(_) => 0,
      Err(e) => {
        set_last_error(&e.to_string());
        -1
      }
    }
  }
}

#[no_mangle]
pub extern "C" fn send_close_connection(client_ptr: *mut LlrpClientWrapper) -> i32 {
  unsafe {
    
    if client_ptr.is_null() {
      set_last_error("Null client pointer");
      return -1;
    }

    let client = &mut *client_ptr;
    match RUNTIME.block_on(client.0.send_close_connection()) {
      Ok(_) => 0,
      Err(e) => {
        set_last_error(&e.to_string());
        -1
      }
    }
  }
}

#[no_mangle]
pub extern "C" fn free_client(client_ptr: *mut LlrpClientWrapper) -> i32 {
  if !client_ptr.is_null() {

    unsafe {
      let _ = Box::from_raw(client_ptr);
    }
    
    0
  } else {
    set_last_error("Null client pointer");
    return -1;
  }
}

#[no_mangle]
pub extern "C" fn free_string(string_ptr: *mut c_char) -> i32 {
  if !string_ptr.is_null() {
    
    unsafe {
      let _ = CString::from_raw(string_ptr);
    }

    0
  } else {
    set_last_error("Null string pointer");
    return -1;
  }
}

#[no_mangle]
pub extern "C" fn get_last_error() -> *const c_char {
  let error = LAST_ERROR.lock().unwrap();
  match &*error {
    Some(err) => CString::new(err.clone()).unwrap().into_raw(),
    None => ptr::null(),
  }
}

fn set_last_error(err: &str) {
  *LAST_ERROR.lock().unwrap() = Some(err.to_string());
}