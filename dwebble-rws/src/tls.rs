//! TLS configuration using rustls with ring

use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;

use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::ServerConfig;
use tokio_rustls::TlsAcceptor;

/// TLS configuration for the server
pub struct TlsConfig {
    pub acceptor: TlsAcceptor,
}

impl TlsConfig {
    /// Create TLS config from certificate and private key PEM files
    pub fn from_pem_files(cert_path: &str, key_path: &str) -> Result<Self, TlsError> {
        let certs = load_certs(cert_path)?;
        let key = load_private_key(key_path)?;

        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| TlsError::Config(e.to_string()))?;

        Ok(Self {
            acceptor: TlsAcceptor::from(Arc::new(config)),
        })
    }
}

/// Load certificates from a PEM file
fn load_certs(path: &str) -> Result<Vec<CertificateDer<'static>>, TlsError> {
    let file = File::open(path).map_err(|e| TlsError::CertLoad(e.to_string()))?;
    let mut reader = BufReader::new(file);

    rustls_pemfile::certs(&mut reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| TlsError::CertLoad(e.to_string()))
}

/// Load private key from a PEM file
fn load_private_key(path: &str) -> Result<PrivateKeyDer<'static>, TlsError> {
    let file = File::open(path).map_err(|e| TlsError::KeyLoad(e.to_string()))?;
    let mut reader = BufReader::new(file);

    loop {
        match rustls_pemfile::read_one(&mut reader)
            .map_err(|e| TlsError::KeyLoad(e.to_string()))?
        {
            Some(rustls_pemfile::Item::Pkcs1Key(key)) => {
                return Ok(PrivateKeyDer::Pkcs1(key));
            }
            Some(rustls_pemfile::Item::Pkcs8Key(key)) => {
                return Ok(PrivateKeyDer::Pkcs8(key));
            }
            Some(rustls_pemfile::Item::Sec1Key(key)) => {
                return Ok(PrivateKeyDer::Sec1(key));
            }
            None => break,
            _ => continue,
        }
    }

    Err(TlsError::KeyLoad("No private key found in file".to_string()))
}

#[derive(Debug)]
pub enum TlsError {
    CertLoad(String),
    KeyLoad(String),
    Config(String),
}

impl std::fmt::Display for TlsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TlsError::CertLoad(e) => write!(f, "Failed to load certificate: {}", e),
            TlsError::KeyLoad(e) => write!(f, "Failed to load private key: {}", e),
            TlsError::Config(e) => write!(f, "TLS configuration error: {}", e),
        }
    }
}

impl std::error::Error for TlsError {}
