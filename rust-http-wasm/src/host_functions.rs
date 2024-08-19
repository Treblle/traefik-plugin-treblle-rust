use std::ffi::CString;

#[link(wasm_import_module = "http_handler")]
extern "C" {
    fn log(level: i32, message: *const u8, message_len: u32);
    // Add other external functions here...
}

pub const LOG_LEVEL_ERROR: i32 = 2;
pub const LOG_LEVEL_WARN: i32 = 1;
pub const LOG_LEVEL_INFO: i32 = 0;
pub const LOG_LEVEL_DEBUG: i32 = -1;

pub fn log_message(level: i32, message: &str) {
    let c_message = CString::new(message).unwrap();
    unsafe {
        log(
            level,
            c_message.as_ptr() as *const u8,
            c_message.as_bytes().len() as u32,
        );
    }
}
