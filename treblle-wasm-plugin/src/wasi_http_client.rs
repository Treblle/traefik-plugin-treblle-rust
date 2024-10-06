use std::io::{self, Read, Write};
use std::time::{Duration, Instant};

#[cfg(feature = "wasm")]
use wasmedge_wasi_socket::{TcpStream, Shutdown};

use std::sync::atomic::{AtomicUsize, Ordering};
use crate::error::{Result, TreblleError};
use crate::logger::{log, LogLevel};
use rustls::{ClientConfig, ClientConnection, OwnedTrustAnchor, RootCertStore, ServerName, StreamOwned};
use std::sync::Arc;
use std::fs::File;
use std::io::BufReader;

pub struct WasiHttpClient {
    treblle_api_urls: Vec<String>,
    current_url_index: AtomicUsize,
}

impl WasiHttpClient {
    pub fn new(treblle_api_urls: Vec<String>) -> Self {
        Self {
            treblle_api_urls,
            current_url_index: AtomicUsize::new(0),
        }
    }

    fn get_next_url(&self) -> String {
        let index = self.current_url_index.fetch_add(1, Ordering::SeqCst) % self.treblle_api_urls.len();
        self.treblle_api_urls[index].clone()
    }

    #[cfg(feature = "wasm")]
    pub fn post(&self, payload: &[u8], api_key: &str) -> Result<()> {
        log(LogLevel::Debug, "Entering post method");
        let url = self.get_next_url();
        log(LogLevel::Debug, &format!("Got URL: {}", url));

        let (host, port, path) = match Self::parse_url(&url) {
            Ok((h, p, path)) => {
                log(LogLevel::Debug, &format!("Parsed URL - host: {}, port: {}, path: {}", h, p, path));
                (h, p, path)
            },
            Err(e) => {
                log(LogLevel::Error, &format!("Failed to parse URL: {}", e));
                return Err(e);
            }
        };

        let start_time = Instant::now();
        log(LogLevel::Debug, &format!("Connecting to host: {}, port: {}", host, port));

        let mut stream = self.connect_with_timeout(&format!("{}:{}", host, port), Duration::from_secs(10))?;
        log(LogLevel::Debug, &format!("Connection established in {:?}", start_time.elapsed()));

        stream.set_nonblocking(true)?;

        // Set up TLS
        let tls_config = self.create_tls_config()?;
        let server_name = ServerName::try_from(host.as_str())
            .map_err(|_| TreblleError::InvalidHostname)?;
        let conn = ClientConnection::new(Arc::new(tls_config), server_name)?;
        let mut tls_stream = StreamOwned::new(conn, stream);

        let request = self.build_post_request(&host, &path, payload, api_key);
        log(LogLevel::Debug, &format!("Sending request:\n{}", request));

        let write_start = Instant::now();
        self.write_with_timeout(&mut tls_stream, request.as_bytes(), Duration::from_secs(10))?;
        log(LogLevel::Debug, &format!("Request sent in {:?}", write_start.elapsed()));

        let mut response = Vec::new();
        self.read_response(&mut tls_stream, &mut response, Duration::from_secs(10))?;

        let elapsed = start_time.elapsed();
        log(LogLevel::Debug, &format!("Full request-response cycle completed in {:?}", elapsed));

        self.handle_response(&response)
    }

    fn create_tls_config(&self) -> Result<ClientConfig> {
        let mut root_store = RootCertStore::empty();

        // Try to load custom root CA
        match Self::load_custom_root_ca() {
            Ok(custom_ca) => {
                root_store.add(&custom_ca)?;
                log(LogLevel::Info, "Custom root CA loaded successfully");
            }
            Err(_) => {
                log(LogLevel::Info, "Failed to load custom root CA, falling back to webpki-roots");
                root_store.add_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.iter().map(|ta| {
                    OwnedTrustAnchor::from_subject_spki_name_constraints(
                        ta.subject,
                        ta.spki,
                        ta.name_constraints,
                    )
                }));
            }
        }

        let config = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        Ok(config)
    }

    fn load_custom_root_ca() -> Result<rustls::Certificate> {
        let mut file = File::open("/etc/certs/rootCA.pem")?;
        let mut pem = Vec::new();
        file.read_to_end(&mut pem)?;
        let certs = rustls_pemfile::certs(&mut pem.as_slice())
            .filter_map(|result| result.ok())
            .collect::<Vec<_>>();
        certs.into_iter().next()
            .map(|cert_der| rustls::Certificate(cert_der.to_vec()))
            .ok_or_else(|| TreblleError::CertificateError("No certificate found in PEM file".to_string()))
    }

    #[cfg(feature = "wasm")]
    fn connect_with_timeout(&self, addr: &str, timeout: Duration) -> Result<TcpStream> {
        let start = Instant::now();
        while start.elapsed() < timeout {
            match TcpStream::connect(addr) {
                Ok(stream) => return Ok(stream),
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(e) => return Err(TreblleError::Io(e)),
            }
        }
        Err(TreblleError::TimeoutError)
    }

    fn write_with_timeout<W: Write>(&self, writer: &mut W, buf: &[u8], timeout: Duration) -> Result<()> {
        let start = Instant::now();
        let mut written = 0;

        while written < buf.len() {
            if start.elapsed() > timeout {
                return Err(TreblleError::Io(io::Error::new(io::ErrorKind::TimedOut, "Write timed out")));
            }

            match writer.write(&buf[written..]) {
                Ok(n) => written += n,
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    std::thread::yield_now();
                }
                Err(e) => return Err(TreblleError::Io(e)),
            }
        }

        Ok(())
    }

    fn read_response<R: Read>(&self, reader: &mut R, response: &mut Vec<u8>, timeout: Duration) -> Result<()> {
        let start = Instant::now();
        let mut buf = [0; 4096];

        loop {
            if start.elapsed() > timeout {
                return Err(TreblleError::Io(io::Error::new(io::ErrorKind::TimedOut, "Read timed out")));
            }

            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    response.extend_from_slice(&buf[..n]);
                    if self.is_response_complete(response) {
                        break;
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    std::thread::yield_now();
                }
                Err(e) => return Err(TreblleError::Io(e)),
            }
        }

        log(LogLevel::Debug, &format!("Response read in {:?}", start.elapsed()));
        Ok(())
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

    fn parse_url(url: &str) -> Result<(String, u16, String)> {
        log(LogLevel::Debug, &format!("Parsing URL: {}", url));
        let url = url::Url::parse(url).map_err(|e| TreblleError::InvalidUrl(e.to_string()))?;
        let host = url.host_str().ok_or_else(|| TreblleError::InvalidUrl("No host in URL".to_string()))?.to_string();
        let port = url.port().unwrap_or_else(|| if url.scheme() == "https" { 443 } else { 80 });
        let path = url.path().to_string();

        log(LogLevel::Debug, &format!("Parsed URL - host: {}, port: {}, path: {}", host, port, path));
        Ok((host, port, path))
    }

    fn build_post_request(&self, host: &str, path: &str, payload: &[u8], api_key: &str) -> String {
        format!(
            "POST {} HTTP/1.1\r\n\
             Host: {}\r\n\
             Content-Type: application/json\r\n\
             X-Api-Key: {}\r\n\
             Content-Length: {}\r\n\
             \r\n\
             {}",
            path,
            host,
            api_key,
            payload.len(),
            String::from_utf8_lossy(payload)
        )
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
}