//! Certificate handling module for the Treblle middleware.
//!
//! This module provides functionality for loading root certificates,
//! either from a custom file specified in the configuration or from
//! the webpki-roots bundle.

use rustls::{OwnedTrustAnchor, RootCertStore};
use std::fs::File;
use std::io::BufReader;
use crate::error::{Result, TreblleError};
use crate::logger::{log, LogLevel};
use crate::CONFIG;

/// Loads root certificates into the provided `RootCertStore`.
///
/// This function first attempts to load custom certificates if a path
/// is specified in the configuration. If that fails or no path is
/// specified, it falls back to loading the webpki-roots bundle.
///
/// # Arguments
///
/// * `root_store` - A mutable reference to the `RootCertStore` to load certificates into.
///
/// # Returns
///
/// A `Result` indicating success or failure of the certificate loading process.
///
/// # Errors
///
/// This function will return an error if:
/// - The custom certificate file cannot be opened or read.
/// - The custom certificates cannot be parsed.
/// - The certificates cannot be added to the `RootCertStore`.
pub fn load_root_certs(root_store: &mut RootCertStore) -> Result<()> {
    if let Some(ca_path) = &CONFIG.root_ca_path {
        match load_custom_certificates(root_store, ca_path) {
            Ok(_) => {
                log(LogLevel::Debug, "Custom root CA loaded successfully");
                return Ok(());
            }
            Err(e) => {
                log(LogLevel::Error, &format!("Failed to load custom root CA: {}. Falling back to webpki-roots.", e));
            }
        }
    }

    load_webpki_roots(root_store)?;
    log(LogLevel::Debug, "Webpki root certificates loaded successfully");
    Ok(())
}

/// Loads custom certificates from a specified file path.
///
/// # Arguments
///
/// * `root_store` - A mutable reference to the `RootCertStore` to load certificates into.
/// * `ca_path` - The file path of the custom root CA certificate.
///
/// # Returns
///
/// A `Result` indicating success or failure of the custom certificate loading process.
///
/// # Errors
///
/// This function will return an error if:
/// - The certificate file cannot be opened or read.
/// - The certificates cannot be parsed.
/// - The certificates cannot be added to the `RootCertStore`.
fn load_custom_certificates(root_store: &mut RootCertStore, ca_path: &str) -> Result<()> {
    let file = File::open(ca_path).map_err(|e| {
        log(LogLevel::Error, &format!("Failed to open custom root CA file: {}", e));
        TreblleError::Certificate(format!("Failed to open custom root CA file: {}", e))
    })?;

    let mut reader = BufReader::new(file);
    let certs = rustls_pemfile::certs(&mut reader)
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| {
            log(LogLevel::Error, &format!("Failed to parse custom root CA file: {}", e));
            TreblleError::Certificate(format!("Failed to parse custom root CA file: {}", e))
        })?;

    if certs.is_empty() {
        log(LogLevel::Error, "No certificates found in the custom root CA file");
        return Err(TreblleError::Certificate("No certificates found in the custom root CA file".to_string()));
    }

    for cert in certs {
        root_store.add(&rustls::Certificate(cert.to_vec())).map_err(|e| {
            log(LogLevel::Error, &format!("Failed to add custom root CA to store: {}", e));
            TreblleError::Certificate(format!("Failed to add custom root CA to store: {}", e))
        })?;
    }

    Ok(())
}

/// Loads the default webpki-roots certificate bundle.
///
/// # Arguments
///
/// * `root_store` - A mutable reference to the `RootCertStore` to load certificates into.
///
/// # Returns
///
/// A `Result` indicating success or failure of the webpki-roots loading process.
fn load_webpki_roots(root_store: &mut RootCertStore) -> Result<()> {
    root_store.add_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.iter().map(|ta| {
        OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject,
            ta.spki,
            ta.name_constraints,
        )
    }));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_webpki_roots() {
        let mut root_store = RootCertStore::empty();
        let result = load_webpki_roots(&mut root_store);
        assert!(result.is_ok());
        assert!(root_store.len() > 0);
    }
}