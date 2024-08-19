use std::ffi::CString;

#[link(wasm_import_module = "http_handler")]
extern "C" {
    fn log(level: i32, message: *const u8, message_len: u32);
    fn get_config(buf: *const u8, buf_limit: i32) -> i32;
    // Add other external functions here...
}

pub const LOG_LEVEL_ERROR: i32 = 2;
pub const LOG_LEVEL_WARN: i32 = 1;
pub const LOG_LEVEL_INFO: i32 = 0;
pub const LOG_LEVEL_DEBUG: i32 = -1;

pub fn middleware_log(level: i32, message: &str) {
    let c_message = CString::new(message).unwrap();
    unsafe {
        log(
            level,
            c_message.as_ptr() as *const u8,
            c_message.as_bytes().len() as u32,
        );
    }
}

pub fn middleware_get_config() -> String {
    let buffer: [u8; 2048] = [0; 2048];
    let len = unsafe { get_config(buffer.as_ptr(), buffer.len() as i32) };
    if len < 0 {
        String::new()
    } else {
        String::from_utf8_lossy(&buffer[..len as usize]).to_string()
    }
}
