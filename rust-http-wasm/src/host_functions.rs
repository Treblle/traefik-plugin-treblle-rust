use crate::error::{Result, TreblleError};
use core::str;
use std::ffi::CString;
use crate::constants::{LOG_LEVEL_ERROR, LOG_LEVEL_INFO};

#[link(wasm_import_module = "http_handler")]
extern "C" {
    fn log(level: i32, message: *const u8, message_len: u32);
    fn enable_features(features: u32) -> u32;
    fn get_config(buf: *mut u8, buf_limit: i32) -> i32;

    fn get_method(buf: *mut u8, buf_limit: i32) -> i32;
    fn get_uri(ptr: *mut u8, buf_limit: u32) -> i32;
    fn get_protocol_version(buf: *mut u8, buf_limit: i32) -> i32;
    fn get_header_names(header_kind: u32, buf: *mut u8, buf_limit: i32) -> i64;
    fn get_header_values(header_kind: u32, name_ptr: *const u8, name_len: u32, buf: *mut u8, buf_limit: i32) -> i64;
    fn read_body(body_kind: u32, ptr: *mut u8, buf_limit: u32) -> i64;
    fn get_status_code() -> u32;
}

pub fn host_log(level: i32, message: &str) {
    let sanitized_message = message.replace('\0', "");
    if let Ok(c_message) = CString::new(sanitized_message) {
        unsafe {
            log(level, c_message.as_ptr() as *const u8, c_message.as_bytes().len() as u32);
        }
    }
}

pub fn host_enable_features(features: u32) -> u32 {
    unsafe { enable_features(features) }
}

pub fn host_get_config() -> Result<String> {
    read_from_buffer(|buf, buf_limit| unsafe { get_config(buf, buf_limit) })
}

pub fn host_get_method() -> Result<String> {
    read_from_buffer(|buf, buf_limit| unsafe { get_method(buf, buf_limit) })
}

pub fn host_get_uri() -> Result<String> {
    read_from_buffer(|buf, buf_limit| unsafe { get_uri(buf, buf_limit as u32) })
}

pub fn host_get_protocol_version() -> Result<String> {
    read_from_buffer(|buf, buf_limit| unsafe { get_protocol_version(buf, buf_limit) })
}

pub fn host_get_header_names(header_kind: u32) -> Result<String> {
    read_from_buffer(|buf, buf_limit| unsafe {
        get_header_names(header_kind, buf, buf_limit as i32) as i32
    })
}

pub fn host_get_header_values(header_kind: u32, name: &str) -> Result<String> {
    let sanitized_name = name.replace('\0', "");
    let c_name = CString::new(sanitized_name)
        .map_err(|e| TreblleError::HostFunction(format!("Invalid header name: {}", e)))?;

    read_from_buffer(|buf, buf_limit| unsafe {
        get_header_values(
            header_kind,
            c_name.as_ptr() as *const u8,
            c_name.as_bytes().len() as u32,
            buf,
            buf_limit,
        ) as i32
    })
}

pub fn host_read_body(body_kind: u32) -> Result<Vec<u8>> {
    host_log(LOG_LEVEL_INFO, "Starting to read body");

    let mut buffer = Vec::with_capacity(4096);
    let read = unsafe { read_body(body_kind, buffer.as_mut_ptr(), 4096) };

    if read < 0 {
        host_log(LOG_LEVEL_ERROR, &format!("Error reading body: {}", read));
        return Err(TreblleError::HostFunction("Error reading body".to_string()));
    }

    unsafe { buffer.set_len(read as usize); }

    host_log(LOG_LEVEL_INFO, &format!("Successfully read {} bytes from body", read));
    Ok(buffer)
}

pub fn host_get_status_code() -> u32 {
    unsafe { get_status_code() }
}

fn read_from_buffer<F: Fn(*mut u8, i32) -> i32>(read_fn: F) -> Result<String> {
    let mut buffer = vec![0u8; 4096];
    let len = read_fn(buffer.as_mut_ptr(), buffer.len() as i32);
    if len < 0 {
        Err(TreblleError::HostFunction("Failed to read from buffer".to_string()))
    } else {
        buffer.truncate(len as usize);
        String::from_utf8(buffer).map_err(|e| TreblleError::HostFunction(e.to_string()))
    }
}