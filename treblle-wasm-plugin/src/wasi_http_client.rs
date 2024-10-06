use std::io::{self, Read, Write};
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[cfg(feature = "wasm")]
use wasmedge_wasi_socket::{TcpStream, Shutdown};

use rustls::{ClientConfig, ClientConnection, OwnedTrustAnchor, RootCertStore, ServerName, StreamOwned};
use url::Url;

use crate::error::{Result, TreblleError};
use crate::logger::{log, LogLevel};
use crate::certs::load_root_certs;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// A client for making HTTP requests in a WASM WASI environment.
pub struct WasiHttpClient {
    treblle_api_urls: Vec<String>,
    current_url_index: AtomicUsize,
    client_config: ClientConfig,
}

impl WasiHttpClient {
    /// Creates a new `WasiHttpClient` instance.
    ///
    /// # Arguments
    ///
    /// * `treblle_api_urls` - A vector of Treblle API URLs to use for requests.
    ///
    /// # Returns
    ///
    /// A new `WasiHttpClient` instance.
    pub fn new(treblle_api_urls: Vec<String>) -> Self {
        let client_config = Self::create_tls_config()
            .expect("Failed to create TLS config");

        Self {
            treblle_api_urls,
            current_url_index: AtomicUsize::new(0),
            client_config,
        }
    }

    fn get_next_url(&self) -> String {
        let index = self.current_url_index.fetch_add(1, Ordering::SeqCst) % self.treblle_api_urls.len();
        self.treblle_api_urls[index].clone()
    }

    /// Sends a POST request to the Treblle API.
    ///
    /// # Arguments
    ///
    /// * `payload` - The JSON payload to send.
    /// * `api_key` - The API key for authentication.
    ///
    /// # Returns
    ///
    /// A `Result` containing the response as a string if successful.
    #[cfg(feature = "wasm")]
    pub fn post(&self, payload: &[u8], api_key: &str) -> Result<()> {
        log(LogLevel::Debug, "Entering post method");
        let url = self.get_next_url();
        log(LogLevel::Debug, &format!("Got URL: {}", url));

        let parsed_url = Url::parse(&url).map_err(|e| TreblleError::InvalidUrl(e.to_string()))?;
        let host = parsed_url.host_str().ok_or_else(|| TreblleError::InvalidUrl("No host in URL".to_string()))?;
        let port = parsed_url.port_or_known_default().ok_or_else(|| TreblleError::InvalidUrl("Invalid port".to_string()))?;
        let path = parsed_url.path();

        log(LogLevel::Debug, &format!("Parsed URL - host: {}, port: {}, path: {}", host, port, path));

        let start_time = Instant::now();
        let timeout = Duration::from_secs(30); // Default timeout
        let stream = self.connect_with_timeout(host, port, timeout)?;

        if parsed_url.scheme() == "https" {
            let tls_stream = self.establish_tls(stream, host)?;
            self.send_request(tls_stream, host, path, payload, api_key, start_time, timeout)
        } else {
            self.send_request(stream, host, path, payload, api_key, start_time, timeout)
        }
    }

    #[cfg(feature = "wasm")]
    fn connect_with_timeout(&self, host: &str, port: u16, timeout: Duration) -> Result<TcpStream> {
        let start = Instant::now();
        while start.elapsed() < timeout {
            match TcpStream::connect((host, port)) {
                Ok(stream) => {
                    log(LogLevel::Debug, &format!("Connection established in {:?}", start.elapsed()));
                    return Ok(stream);
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(e) => return Err(TreblleError::Io(e)),
            }
        }
        Err(TreblleError::TimeoutError)
    }

    #[cfg(feature = "wasm")]
    fn establish_tls(&self, tcp_stream: TcpStream, host: &str) -> Result<StreamOwned<ClientConnection, TcpStream>> {
        let config = Arc::new(self.client_config.clone());
        let server_name = ServerName::try_from(host)
            .map_err(|_| TreblleError::InvalidHostname)?;
        let conn = ClientConnection::new(config, server_name)?;
        Ok(StreamOwned::new(conn, tcp_stream))
    }

    fn send_request<T: Read + Write>(&self, mut stream: T, host: &str, path: &str, payload: &[u8], api_key: &str, start_time: Instant, timeout: Duration) -> Result<()> {
        self.write_request(&mut stream, host, path, payload, api_key)?;
        let response = self.read_response(&mut stream, start_time, timeout)?;
        self.handle_response(&response)
    }

    fn write_request<T: Write>(&self, stream: &mut T, host: &str, path: &str, payload: &[u8], api_key: &str) -> Result<()> {
        let request = format!(
            "POST {} HTTP/1.1\r\n\
             Host: {}\r\n\
             Content-Type: application/json\r\n\
             X-Api-Key: {}\r\n\
             Content-Length: {}\r\n\
             \r\n",
            path, host, api_key, payload.len()
        );
        stream.write_all(request.as_bytes())?;
        stream.write_all(payload)?;
        stream.flush()?;
        Ok(())
    }

    fn read_response<T: Read>(&self, stream: &mut T, start_time: Instant, timeout: Duration) -> Result<Vec<u8>> {
        let mut response = Vec::new();
        let mut buffer = [0; 1024];

        while start_time.elapsed() < timeout {
            match stream.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    response.extend_from_slice(&buffer[..n]);
                    if self.is_response_complete(&response) {
                        break;
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => continue,
                Err(e) => return Err(TreblleError::Io(e)),
            }
        }

        if response.is_empty() {
            return Err(TreblleError::TimeoutError);
        }

        Ok(response)
    }

    fn is_response_complete(&self, response: &[u8]) -> bool {
        if let Some(headers_end) = response.windows(4).position(|window| window == b"\r\n\r\n") {
            if let Some(content_length) = self.get_content_length(response) {
                return response.len() >= headers_end + 4 + content_length;
            }
        }
        false
    }

    fn get_content_length(&self, response: &[u8]) -> Option<usize> {
        let headers = String::from_utf8_lossy(response);
        headers.lines()
            .find(|line| line.to_lowercase().starts_with("content-length:"))
            .and_then(|line| line.split(':').nth(1))
            .and_then(|len| len.trim().parse().ok())
    }

    fn handle_response(&self, response: &[u8]) -> Result<()> {
        let response_str = String::from_utf8_lossy(response);
        log(LogLevel::Debug, &format!("Raw response: {}", response_str));

        // Log headers
        if let Some(headers_end) = response_str.find("\r\n\r\n") {
            let headers = &response_str[..headers_end];
            log(LogLevel::Debug, "Response headers:");
            for header in headers.lines() {
                log(LogLevel::Debug, header);
            }
        }

        // Check status code
        if let Some(status_line) = response_str.lines().next() {
            if !status_line.contains("200 OK") {
                log(LogLevel::Error, &format!("HTTP error: {}", status_line));
                return Err(TreblleError::Http(format!("HTTP error: {}", status_line)));
            }
        }

        // Log body
        if let Some(body_start) = response_str.find("\r\n\r\n") {
            let body = &response_str[body_start + 4..];
            log(LogLevel::Debug, &format!("Response body: {}", body));
        }

        log(LogLevel::Debug, "Successfully sent data");
        Ok(())
    }

    fn create_tls_config() -> Result<ClientConfig> {
        let mut root_store = RootCertStore::empty();
        load_root_certs(&mut root_store)?;

        let config = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_write_request() {
        let client = WasiHttpClient::new(vec!["https://api.treblle.com".to_string()]);
        let mut buffer = Vec::new();
        client.write_request(&mut buffer, "api.treblle.com", "/v1/log", b"{\"key\":\"value\"}", "test-api-key")
            .expect("Failed to write request");

        let request = String::from_utf8(buffer).expect("Invalid UTF-8");
        assert!(request.starts_with("POST /v1/log HTTP/1.1\r\n"));
        assert!(request.contains("Host: api.treblle.com\r\n"));
        assert!(request.contains("Content-Type: application/json\r\n"));
        assert!(request.contains("X-Api-Key: test-api-key\r\n"));
        assert!(request.ends_with("\r\n\r\n{\"key\":\"value\"}"));
    }

    #[test]
    fn test_read_response() {
        let client = WasiHttpClient::new(vec!["https://api.treblle.com".to_string()]);
        let response_data = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 20\r\n\r\n{\"status\":\"success\"}";
        let mut cursor = Cursor::new(response_data);

        let result = client.read_response(&mut cursor, Instant::now(), Duration::from_secs(1))
            .expect("Failed to read response");

        assert_eq!(result, response_data.as_bytes());
    }
}