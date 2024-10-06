use std::io::{self, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use std::collections::VecDeque;

#[cfg(feature = "wasm")]
use wasmedge_wasi_socket::{TcpStream, Shutdown};

use rustls::{ClientConfig, ClientConnection, RootCertStore, ServerName, StreamOwned};
use url::Url;

use crate::error::{Result, TreblleError};
use crate::logger::{log, LogLevel};
use crate::certs::load_root_certs;
use crate::CONFIG;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_POOL_SIZE: usize = 10;
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(60);

#[cfg(feature = "wasm")]
type TlsStream = StreamOwned<ClientConnection, TcpStream>;

struct PooledConnection {
    #[cfg(feature = "wasm")]
    stream: TlsStream,
    last_used: Instant,
}

pub struct WasiHttpClient {
    treblle_api_urls: Vec<String>,
    current_url_index: AtomicUsize,
    client_config: ClientConfig,
    connection_pool: VecDeque<PooledConnection>,
}

impl WasiHttpClient {
    pub fn new(treblle_api_urls: Vec<String>) -> Result<Self> {
        let client_config = Self::create_tls_config()?;
        
        Ok(Self {
            treblle_api_urls,
            current_url_index: AtomicUsize::new(0),
            client_config,
            connection_pool: VecDeque::new(),
        })
    }

    fn get_next_url(&self) -> String {
        let index = self.current_url_index.fetch_add(1, Ordering::SeqCst) % self.treblle_api_urls.len();
        self.treblle_api_urls[index].clone()
    }

    #[cfg(feature = "wasm")]
    pub fn post(&mut self, payload: &[u8], api_key: &str) -> Result<()> {
        let url = self.get_next_url();
        let parsed_url = Url::parse(&url).map_err(|e| TreblleError::InvalidUrl(e.to_string()))?;
        let host = parsed_url.host_str().ok_or_else(|| TreblleError::InvalidUrl("No host in URL".to_string()))?;
        let port = parsed_url.port_or_known_default().ok_or_else(|| TreblleError::InvalidUrl("Invalid port".to_string()))?;
        let path = parsed_url.path();

        let mut stream = self.get_connection(host, port)?;

        let request = self.create_request(host, path, payload, api_key);
        let mut full_request = request.into_bytes();
        full_request.extend_from_slice(payload);

        self.send_non_blocking(&mut stream, &full_request)?;

        self.return_connection(stream);

        Ok(())
    }

    #[cfg(feature = "wasm")]
    fn get_connection(&mut self, host: &str, port: u16) -> Result<TlsStream> {
        // Remove expired connections
        self.connection_pool.retain(|conn| conn.last_used.elapsed() < CONNECTION_TIMEOUT);

        if let Some(mut pooled_conn) = self.connection_pool.pop_front() {
            pooled_conn.last_used = Instant::now();
            return Ok(pooled_conn.stream);
        }

        // Create a new connection if the pool is empty
        let stream = TcpStream::connect((host, port))?;
        stream.set_nonblocking(true)?;

        let server_name = ServerName::try_from(host)
            .map_err(|_| TreblleError::InvalidHostname(host.to_string()))?;
        let client = ClientConnection::new(std::sync::Arc::new(self.client_config.clone()), server_name)
            .map_err(TreblleError::Tls)?;
        Ok(StreamOwned::new(client, stream))
    }

    #[cfg(feature = "wasm")]
    fn return_connection(&mut self, stream: TlsStream) {
        if self.connection_pool.len() < MAX_POOL_SIZE {
            self.connection_pool.push_back(PooledConnection {
                stream,
                last_used: Instant::now(),
            });
        }
    }

    fn create_request(&self, host: &str, path: &str, payload: &[u8], api_key: &str) -> String {
        format!(
            "POST {} HTTP/1.1\r\n\
             Host: {}\r\n\
             Content-Type: application/json\r\n\
             X-Api-Key: {}\r\n\
             Content-Length: {}\r\n\
             Connection: keep-alive\r\n\
             \r\n",
            path, host, api_key, payload.len()
        )
    }
    
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