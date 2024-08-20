use crate::error::{Result, TreblleError};
use core::str;
use std::ffi::CString;

use crate::constants::{LOG_LEVEL_ERROR, LOG_LEVEL_INFO};

// External functions from the `http_handler` module exposed by http-wasm-host-go/api/handler
// https://github.com/http-wasm/http-wasm-host-go/blob/main/api/handler/handler.go
#[link(wasm_import_module = "http_handler")]
extern "C" {
    fn log(level: i32, message: *const u8, message_len: u32);
    fn get_config(buf: *mut u8, buf_limit: i32) -> i32;
    fn get_method(buf: *const u8, buf_limit: i32) -> i32;
    fn get_uri(ptr: *const u8, message_len: u32) -> i32;
    fn get_header_names(header_kind: u32, buf: *const u8, buf_limit: i32) -> i64;
    fn get_header_values(
        header_kind: u32,
        name_ptr: *const u8,
        name_len: u32,
        buf: *const u8,
        buf_limit: i32,
    ) -> i64;
    fn read_body(body_kind: u32, ptr: *const u8, buf_limit: u32) -> i64;
    fn write_body(body_kind: u32, ptr: *const u8, message_len: u32);
    fn get_source_addr(buf: *const u8, buf_limit: i32) -> i32;
}

fn read_from_buffer<F: Fn(*const u8, u32) -> i32>(read_fn: F) -> Result<String> {
    let read_buf: [u8; 2048] = [0; 2048];
    let len = read_fn(read_buf.as_ptr(), 2048);

    if len < 0 {
        host_log(
            LOG_LEVEL_ERROR,
            &format!("Failed to read from buffer: {}", len),
        );

        Err(TreblleError::HostFunction(
            "Failed to read from buffer".to_string(),
        ))
    } else {
        let result = str::from_utf8(&read_buf[0..len as usize])
            .map_err(|e| TreblleError::HostFunction(e.to_string()))?
            .to_string();

        host_log(LOG_LEVEL_INFO, &format!("Read from buffer: {} bytes", len));

        Ok(result)
    }
}

// Update the host_log function to handle potential errors
pub fn host_log(level: i32, message: &str) {
    // Remove null characters from the message
    let sanitized_message = message.replace('\0', "");

    if let Ok(c_message) = CString::new(sanitized_message) {
        unsafe {
            log(
                level,
                c_message.as_ptr() as *const u8,
                c_message.as_bytes().len() as u32,
            );
        }
    } else {
        // If we can't create a CString, log a fallback message
        let fallback = CString::new("Error logging message: contains null characters").unwrap();

        unsafe {
            log(
                level,
                fallback.as_ptr() as *const u8,
                fallback.as_bytes().len() as u32,
            );
        }
    }
}

pub fn host_get_config() -> Result<String> {
    read_from_buffer(|buf, buf_limit| unsafe { get_config(buf as *mut u8, buf_limit as i32) })
}
pub fn host_read_request_body() -> Result<String> {
    read_from_buffer(|buf, buf_limit| unsafe { read_body(0, buf as *mut u8, buf_limit) as i32 })
}

pub fn host_write_request_body(body: &[u8]) -> Result<()> {
    unsafe {
        write_body(0, body.as_ptr(), body.len() as u32);
    }

    host_log(
        LOG_LEVEL_INFO,
        &format!("Wrote {} bytes back to request body", body.len()),
    );

    Ok(())
}

pub fn host_get_method() -> Result<String> {
    read_from_buffer(|buf, buf_limit| unsafe { get_method(buf, buf_limit as i32) })
}

pub fn host_get_uri() -> Result<String> {
    read_from_buffer(|buf, buf_limit| unsafe { get_uri(buf, buf_limit as u32) })
}

pub fn host_get_source_addr() -> Result<String> {
    read_from_buffer(|buf, buf_limit| unsafe { get_source_addr(buf, buf_limit as i32) })
}

pub fn host_get_header_names(header_kind: u32) -> Result<String> {
    let result = read_from_buffer(|buf, buf_limit| unsafe {
        get_header_names(header_kind, buf, buf_limit as i32) as i32
    });

    match result {
        Ok(names) => {
            host_log(LOG_LEVEL_INFO, &format!("Got header names: {}", names));
            Ok(names)
        }
        Err(e) => {
            host_log(
                LOG_LEVEL_ERROR,
                &format!("Failed to get header names: {}", e),
            );
            Err(e)
        }
    }
}

pub fn host_get_header_values(header_kind: u32, name: &str) -> Result<String> {
    // Remove null characters from the header name
    let sanitized_name = name.replace('\0', "");

    // Create a CString, ignoring null characters
    let c_name = CString::new(sanitized_name)
        .map_err(|e| TreblleError::HostFunction(format!("Invalid header name: {}", e)))?;

    let result = read_from_buffer(|buf, buf_limit| unsafe {
        get_header_values(
            header_kind,
            c_name.as_ptr() as *const u8,
            c_name.as_bytes().len() as u32,
            buf,
            buf_limit as i32,
        ) as i32
    });

    match result {
        Ok(values) => {
            host_log(
                LOG_LEVEL_INFO,
                &format!("Got header values for {}: {}", name, values),
            );
            Ok(values)
        }
        Err(e) => {
            host_log(
                LOG_LEVEL_ERROR,
                &format!("Failed to get header values for {}: {}", name, e),
            );
            Err(e)
        }
    }
}
