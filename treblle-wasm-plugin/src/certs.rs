use std::fs::File;
use std::io::Read;
use crate::logger::{log, LogLevel};

pub fn load_root_certificate() -> Result<String, std::io::Error> {
    log(LogLevel::Debug, "Attempting to load root certificate");

    let cert_paths = vec![
        "/etc/certs/rootCA.pem",
        "/etc/rootCA.pem",
        "/etc/certs/ca.crt",
        "/etc/ca.crt",
    ];

    for path in cert_paths {
        log(LogLevel::Debug, &format!("Trying to open certificate file: {}", path));
        match File::open(path) {
            Ok(mut file) => {
                log(LogLevel::Debug, &format!("Successfully opened file: {}", path));
                let mut contents = String::new();
                match file.read_to_string(&mut contents) {
                    Ok(_) => {
                        log(LogLevel::Info, &format!("Successfully read certificate from: {}", path));
                        log(LogLevel::Debug, "Certificate content:");
                        log(LogLevel::Debug, &contents);
                        return Ok(contents);
                    }
                    Err(e) => log(LogLevel::Error, &format!("Failed to read file {}: {}", path, e)),
                }
            }
            Err(e) => log(LogLevel::Error, &format!("Failed to open file {}: {}", path, e)),
        }
    }

    log(LogLevel::Error, "Failed to load root certificate from any known location");
    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Root certificate not found"))
}