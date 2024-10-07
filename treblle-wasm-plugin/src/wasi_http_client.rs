//! HTTP client implementation for WASI environments
//!
//! This module provides an HTTP client implementation specifically designed for
//! WebAssembly System Interface (WASI) environments. Minimalistic, custom-built for this middleware.
//! It supports connection pooling, TLS connections, and non-blocking I/O operations.

use std::collections::VecDeque;
use std::io::{self, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use lazy_static::lazy_static;
use rustls::{ClientConfig, ClientConnection, RootCertStore, ServerName, StreamOwned};
use url::Url;

#[cfg(feature = "wasm")]
use wasmedge_wasi_socket::TcpStream;

use crate::certs::load_root_certs;
use crate::error::{Result, TreblleError};
use crate::logger::{log, LogLevel};

/// Timeout duration for connection attempts
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
/// Timeout duration for idle connections in the pool
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(60);

/// Maximum number of connections to keep in the pool
const MAX_POOL_SIZE: usize = 50;

#[cfg(feature = "wasm")]
type TlsStream = StreamOwned<ClientConnection, TcpStream>;

/// Represents a pooled connection with its last used timestamp
struct PooledConnection {
    #[cfg(feature = "wasm")]
    stream: TlsStream,
    last_used: Instant,
}

lazy_static! {
    /// Global TLS client configuration
    static ref CLIENT_CONFIG: Mutex<Option<Arc<ClientConfig>>> = Mutex::new(None);
}

/// HTTP client for WASI environments with connection pooling
pub struct WasiHttpClient {
    treblle_api_urls: Vec<String>,
    current_url_index: AtomicUsize,
    connection_pool: Mutex<VecDeque<PooledConnection>>,
}

impl WasiHttpClient {
    /// Creates a new `WasiHttpClient` instance
    ///
    /// # Arguments
    ///
    /// * `treblle_api_urls` - A vector of Treblle API URLs to cycle through
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `WasiHttpClient` instance or an error
    pub fn new(treblle_api_urls: Vec<String>) -> Result<Self> {
        Ok(Self {
            treblle_api_urls,
            current_url_index: AtomicUsize::new(0),
            connection_pool: Mutex::new(VecDeque::new()),
        })
    }

    /// Gets the next URL from the list of Treblle API URLs
    fn get_next_url(&self) -> String {
        let index =
            self.current_url_index.fetch_add(1, Ordering::SeqCst) % self.treblle_api_urls.len();
        self.treblle_api_urls[index].clone()
    }

    /// Sends a POST request to the Treblle API
    ///
    /// # Arguments
    ///
    /// * `payload` - The payload to send in the request body
    /// * `api_key` - The API key for authentication
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or containing an error
    #[cfg(feature = "wasm")]
    pub fn post(&self, payload: &[u8], api_key: &str) -> Result<()> {
        let url = self.get_next_url();
        let parsed_url = Url::parse(&url).map_err(|e| TreblleError::InvalidUrl(e.to_string()))?;
        
        let host = parsed_url
            .host_str()
            .ok_or_else(|| TreblleError::InvalidUrl("No host in URL".to_string()))?;
        
        let port = parsed_url
            .port_or_known_default()
            .ok_or_else(|| TreblleError::InvalidUrl("Invalid port".to_string()))?;
        
        let path = parsed_url.path();

        let mut stream = self.get_connection(host, port)?;

        let request = self.create_request(host, path, payload, api_key);
        let mut full_request = request.into_bytes();
        full_request.extend_from_slice(payload);

        self.send_non_blocking(&mut stream, &full_request)?;

        self.return_connection(stream);

        Ok(())
    }

    /// Gets a connection from the pool or creates a new one
    ///
    /// # Arguments
    ///
    /// * `host` - The host to connect to
    /// * `port` - The port to connect to
    ///
    /// # Returns
    ///
    /// A `Result` containing a `TlsStream` or an error
    #[cfg(feature = "wasm")]
    fn get_connection(&self, host: &str, port: u16) -> Result<TlsStream> {
        let mut pool =
            self.connection_pool.lock().map_err(|e| TreblleError::LockError(e.to_string()))?;

        // Remove expired connections
        pool.retain(|conn| conn.last_used.elapsed() < CONNECTION_TIMEOUT);

        if let Some(mut pooled_conn) = pool.pop_front() {
            pooled_conn.last_used = Instant::now();
            return Ok(pooled_conn.stream);
        }

        // Create a new connection if the pool is empty
        let stream = TcpStream::connect((host, port))?;
        stream.set_nonblocking(true)?;

        let server_name = ServerName::try_from(host)
            .map_err(|_| TreblleError::InvalidHostname(host.to_string()))?;
        
        let client = ClientConnection::new(self.get_client_config()?, server_name)
            .map_err(TreblleError::Tls)?;
        
        Ok(StreamOwned::new(client, stream))
    }

    /// Returns a connection to the pool
    ///
    /// # Arguments
    ///
    /// * `stream` - The `TlsStream` to return to the pool
    #[cfg(feature = "wasm")]
    fn return_connection(&self, stream: TlsStream) {
        let mut pool = match self.connection_pool.lock() {
            Ok(guard) => guard,
            Err(e) => {
                log(LogLevel::Error, &format!("Failed to acquire lock for connection pool: {}", e));
                return;
            }
        };

        if pool.len() < MAX_POOL_SIZE {
            pool.push_back(PooledConnection { stream, last_used: Instant::now() });
        }
    }

    /// Creates an HTTP request string
    ///
    /// # Arguments
    ///
    /// * `host` - The host for the request
    /// * `path` - The path for the request
    /// * `payload` - The payload to be sent
    /// * `api_key` - The API key for authentication
    ///
    /// # Returns
    ///
    /// A `String` containing the HTTP request
    fn create_request(&self, host: &str, path: &str, payload: &[u8], api_key: &str) -> String {
        format!(
            "POST {} HTTP/1.1\r\n\
             Host: {}\r\n\
             Content-Type: application/json\r\n\
             X-Api-Key: {}\r\n\
             Content-Length: {}\r\n\
             Connection: keep-alive\r\n\
             \r\n",
            path,
            host,
            api_key,
            payload.len()
        )
    }

    /// Sends data in a non-blocking manner
    ///
    /// # Arguments
    ///
    /// * `writer` - The writer to send data to
    /// * `data` - The data to send
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or containing an error
    fn send_non_blocking<W: Write>(&self, writer: &mut W, data: &[u8]) -> Result<()> {
        let mut written = 0;
        let start = Instant::now();

        while written < data.len() {
            match writer.write(&data[written..]) {
                Ok(n) => written += n,
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    if start.elapsed() > CONNECT_TIMEOUT {
                        return Err(TreblleError::Timeout);
                    }
                    
                    std::thread::sleep(Duration::from_millis(1));
                    
                    continue;
                }
                Err(e) => return Err(TreblleError::Io(e)),
            }
        }

        Ok(())
    }

    /// Creates a new TLS client configuration
    ///
    /// # Returns
    ///
    /// A `Result` containing the `ClientConfig` or an error
    fn create_tls_config() -> Result<ClientConfig> {
        let mut root_store = RootCertStore::empty();
        load_root_certs(&mut root_store)?;

        let config = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        Ok(config)
    }

    /// Gets or initializes the global TLS client configuration
    ///
    /// # Returns
    ///
    /// A `Result` containing an `Arc<ClientConfig>` or an error
    fn get_client_config(&self) -> Result<Arc<ClientConfig>> {
        let mut config_guard =
            CLIENT_CONFIG.lock().map_err(|e| TreblleError::LockError(e.to_string()))?;

        if let Some(config) = config_guard.as_ref() {
            return Ok(config.clone());
        }

        log(LogLevel::Info, "Initializing TLS client configuration");
        let new_config = Arc::new(Self::create_tls_config()?);
        
        *config_guard = Some(new_config.clone());
        
        Ok(new_config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_next_url() {
        let client = WasiHttpClient::new(vec![
            "https://api1.treblle.com".to_string(),
            "https://api2.treblle.com".to_string(),
        ])
        .unwrap();

        assert_eq!(client.get_next_url(), "https://api1.treblle.com");
        assert_eq!(client.get_next_url(), "https://api2.treblle.com");
        assert_eq!(client.get_next_url(), "https://api1.treblle.com");
    }

    #[test]
    fn test_create_request() {
        let client = WasiHttpClient::new(vec!["https://api.treblle.com".to_string()]).unwrap();
        let payload = b"test payload";
        let request = client.create_request("api.treblle.com", "/v1/log", payload, "test_api_key");

        assert!(request.starts_with("POST /v1/log HTTP/1.1\r\n"));
        assert!(request.contains("Host: api.treblle.com\r\n"));
        assert!(request.contains("Content-Type: application/json\r\n"));
        assert!(request.contains("X-Api-Key: test_api_key\r\n"));
        assert!(request.contains(&format!("Content-Length: {}\r\n", payload.len())));
        assert!(request.contains("Connection: keep-alive\r\n"));
    }

    #[test]
    fn test_send_non_blocking() {
        let client = WasiHttpClient::new(vec!["https://api.treblle.com".to_string()]).unwrap();
        let mut buffer = Vec::new();
        let data = b"test data";

        assert!(client.send_non_blocking(&mut buffer, data).is_ok());
        assert_eq!(buffer, data);
    }
}
