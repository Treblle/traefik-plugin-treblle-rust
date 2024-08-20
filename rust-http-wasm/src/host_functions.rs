use anyhow::{Error, Result};
use core::str;
use std::ffi::CString;

/// The `read_from_buffer` is a generic function that takes a function `read_fn` as an argument.
/// The `read_fn` is expected to take a pointer to a byte (`*const u8`) and a 32-bit unsigned integer (`u32`) as arguments, and return a 32-bit integer (`i32`).
/// The `read_from_buffer` function itself returns a `Result` which is either a `String` (in case of success) or an `Error` (in case of failure).
pub fn read_from_buffer<F: Fn(*const u8, u32) -> i32>(read_fn: F) -> Result<String, Error> {
    let read_buf: [u8; 2048] = [0; 2048];
    let len = read_fn(read_buf.as_ptr(), 2048 as u32);

    if len < 0 {
        Err(Error::msg("Failed to read from buffer"))
    } else {
        Ok(str::from_utf8(&read_buf[0..len as usize])
            .unwrap()
            .to_string())
    }
}

#[link(wasm_import_module = "http_handler")]
extern "C" {
    fn log(level: i32, message: *const u8, message_len: u32);
    fn get_config(buf: *mut u8, buf_limit: i32) -> i32;
    fn get_method(buf: *const u8, buf_limit: i32) -> i32;
    fn set_method(ptr: *const u8, message_len: u32);
    fn get_uri(ptr: *const u8, message_len: u32) -> i32;
    fn set_uri(ptr: *const u8, message_len: u32);
    fn get_protocol_version(ptr: *const u8, message_len: u32) -> i32;
    fn add_header_value(
        header_kind: u32,
        name_ptr: *const u8,
        name_len: u32,
        value_ptr: *const u8,
        value_len: u32,
    );
    fn set_header_value(
        header_kind: u32,
        name_ptr: *const u8,
        name_len: u32,
        value_ptr: *const u8,
        value_len: u32,
    );
    fn remove_header(header_kind: u32, name_ptr: *const u8, name_len: u32);
    fn get_header_names(header_kind: u32, buf: *const u8, buf_limit: i32) -> i64;
    fn get_header_values(
        header_kind: u32,
        name_ptr: *const u8,
        name_len: u32,
        buf: *const u8,
        buf_limit: i32,
    ) -> i64;
    fn log_enabled(level: i32) -> i32;
    fn read_body(body_kind: u32, ptr: *const u8, buf_limit: u32) -> i64;
    fn write_body(body_kind: u32, ptr: *const u8, message_len: u32);
    fn get_status_code() -> i32;
    fn set_status_code(code: i32);
    fn enable_features(feature: u32) -> i32;
    fn get_source_addr(buf: *const u8, buf_limit: i32) -> i32;
}

pub fn host_log(level: i32, message: &str) {
    let c_message = CString::new(message).unwrap();
    unsafe {
        log(
            level,
            c_message.as_ptr() as *const u8,
            c_message.as_bytes().len() as u32,
        );
    }
}

pub fn host_get_config() -> String {
    read_from_buffer(|buf, buf_limit| unsafe { get_config(buf as *mut u8, buf_limit as i32) })
        .unwrap_or_else(|_| String::new())
}

pub fn host_read_request_body() -> Result<String, Error> {
    read_from_buffer(|buf, buf_limit| unsafe { read_body(0, buf as *mut u8, buf_limit) as i32 })
}

pub fn host_write_request_body(body: &[u8]) -> Result<(), String> {
    unsafe {
        write_body(0, body.as_ptr(), body.len() as u32);
    }
    Ok(())
}

pub fn host_get_method() -> Result<String, Error> {
    read_from_buffer(|buf, buf_limit| unsafe { get_method(buf, buf_limit as i32) })
}

pub fn host_get_uri() -> Result<String, Error> {
    read_from_buffer(|buf, buf_limit| unsafe { get_uri(buf, buf_limit as u32) })
}

pub fn host_get_protocol_version() -> Result<String, Error> {
    read_from_buffer(|buf, buf_limit| unsafe { get_protocol_version(buf, buf_limit as u32) })
}

pub fn host_get_source_addr() -> Result<String, Error> {
    read_from_buffer(|buf, buf_limit| unsafe { get_source_addr(buf, buf_limit as i32) })
}

pub fn host_get_header_names(header_kind: u32) -> Result<String, Error> {
    read_from_buffer(|buf, buf_limit| unsafe {
        get_header_names(header_kind, buf, buf_limit as i32) as i32
    })
}

pub fn host_get_header_values(header_kind: u32, name: &str) -> Result<String, Error> {
    let c_name = CString::new(name).unwrap();
    read_from_buffer(|buf, buf_limit| unsafe {
        get_header_values(
            header_kind,
            c_name.as_ptr() as *const u8,
            c_name.as_bytes().len() as u32,
            buf,
            buf_limit as i32,
        ) as i32
    })
}
