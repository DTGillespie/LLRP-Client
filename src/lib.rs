use std::os::raw::c_char;
use std::ffi::{CStr, CString};
use std::ptr;
use std::sync::Mutex;
use tokio::runtime::Runtime;
use lazy_static::lazy_static;

mod client;
mod config;
mod llrp;

use client::LlrpClient;

type ROAccessReportCallback = extern "C" fn(report: *const c_char);

lazy_static! {
  static ref RUNTIME: Runtime = Runtime::new().unwrap();
  static ref LAST_ERROR: Mutex<Option<String>> = Mutex::new(None);
  static ref RO_ACCESS_REPORT_CALLBACK: Mutex<Option<ROAccessReportCallback>> = Mutex::new(None);
}

// Opaque pointer to represent `LlrpClient` in C
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

    match RUNTIME.block_on(client.0.send_get_reader_capabilities()) {
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

    match RUNTIME.block_on(client.0.send_get_reader_config()) {
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
pub extern "C" fn set_ro_access_report_callback(callback: ROAccessReportCallback) {
  *RO_ACCESS_REPORT_CALLBACK.lock().unwrap() = Some(callback);
}

#[no_mangle]

pub extern "C" fn await_ro_access_report(client_ptr: *mut LlrpClientWrapper) -> i32 {
  unsafe {

    if client_ptr.is_null() {
      set_last_error("Null client pointer");
      return -1;
    }

    let client = &mut *client_ptr;
    let callback = RO_ACCESS_REPORT_CALLBACK.lock().unwrap();

    if callback.is_none() {
      set_last_error("No ROAccessReport callback registered");
      return -1;
    }

    let callback = callback.unwrap();

    match RUNTIME.block_on(client.0.await_ro_access_report(move | response | async move {

      let report_str = match response.decode() {
        Ok(tag_reports) => tag_reports
          .iter()
          .map(|tag| tag.to_string())
          .collect::<Vec<_>>()
          .join(", "),
        Err(_) => "Error decoding report".to_string()
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