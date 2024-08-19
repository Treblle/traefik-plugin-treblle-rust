use serde_json::Value;
use std::ffi::CString;

#[link(wasm_import_module = "http_handler")]
extern "C" {
    fn http_get(url: *const u8, url_len: u32, response: *mut u8, response_len: *mut u32) -> i32;
    fn http_post(
        url: *const u8,
        url_len: u32,
        body: *const u8,
        body_len: u32,
        response: *mut u8,
        response_len: *mut u32,
    ) -> i32;
}

pub struct Response {
    pub status: i32,
    pub body: Vec<u8>,
}

pub fn get(url: &str) -> Result<Response, String> {
    let c_url = CString::new(url).unwrap();
    let mut response = vec![0u8; 4096];
    let mut response_len = response.len() as u32;

    let status = unsafe {
        http_get(
            c_url.as_ptr() as *const u8,
            c_url.as_bytes().len() as u32,
            response.as_mut_ptr(),
            &mut response_len,
        )
    };

    if status >= 200 && status < 300 {
        response.truncate(response_len as usize);
        Ok(Response {
            status,
            body: response,
        })
    } else {
        Err(format!("HTTP GET request failed with status: {}", status))
    }
}

pub fn post(url: &str, payload: &Value) -> Result<Response, String> {
    let c_url = CString::new(url).unwrap();
    let body = serde_json::to_vec(payload).map_err(|e| e.to_string())?;
    let mut response = vec![0u8; 4096];
    let mut response_len = response.len() as u32;

    let status = unsafe {
        http_post(
            c_url.as_ptr() as *const u8,
            c_url.as_bytes().len() as u32,
            body.as_ptr(),
            body.len() as u32,
            response.as_mut_ptr(),
            &mut response_len,
        )
    };

    if status >= 200 && status < 300 {
        response.truncate(response_len as usize);
        Ok(Response {
            status,
            body: response,
        })
    } else {
        Err(format!("HTTP POST request failed with status: {}", status))
    }
}
