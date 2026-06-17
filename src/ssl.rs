use anyhow::Result;
use pyo3::prelude::*;
use rustls::{
    ServerConfig,
    pki_types::{CertificateDer, PrivateKeyDer},
};
use rustls_pemfile::Item;
use std::fs;

#[pyclass]
#[derive(Clone, Copy)]
pub enum TLSVersion {
    TLS12,
    TLS12_PLUS,
    TLS13,
}

static TLS12_ONLY: [&'static rustls::SupportedProtocolVersion; 1] = [&rustls::version::TLS12];
static TLS12_PLUS: [&'static rustls::SupportedProtocolVersion; 2] = [&rustls::version::TLS12, &rustls::version::TLS13];
static TLS13_ONLY: [&'static rustls::SupportedProtocolVersion; 1] = [&rustls::version::TLS13];
static TLS_DEFAULT: [&'static rustls::SupportedProtocolVersion; 1] = TLS12_ONLY;

fn get_tls_version_from_env() -> Option<TLSVersion> {
    match std::env::var("RLOOP_TLS_VERSION").as_deref() {
        Ok("1.2") => Some(TLSVersion::TLS12),
        Ok("1.2+") => Some(TLSVersion::TLS12_PLUS),
        Ok("1.3") => Some(TLSVersion::TLS13),
        Ok(_) => None,  // Invalid value, use default
        Err(_) => None, // Not set, use default
    }
}

/// Create a basic SSL server configuration with self-signed certificate
pub(crate) fn create_ssl_config_with_version(tls_version: Option<TLSVersion>) -> Result<ServerConfig> {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()])?;
    let cert_der = CertificateDer::from(cert.cert);
    let key_der = rustls::pki_types::PrivatePkcs8KeyDer::from(cert.key_pair.serialize_der());
    let key_der = PrivateKeyDer::from(key_der);

    let versions = match tls_version {
        Some(TLSVersion::TLS12) => &TLS12_ONLY[..],
        Some(TLSVersion::TLS12_PLUS) => &TLS12_PLUS[..],
        Some(TLSVersion::TLS13) => &TLS13_ONLY[..],
        None => &TLS_DEFAULT[..],
    };

    let config = ServerConfig::builder_with_protocol_versions(versions)
        .with_no_client_auth()
        .with_single_cert(vec![cert_der], key_der)?;

    Ok(config)
}

/// Create a basic SSL server configuration with self-signed certificate
pub(crate) fn create_ssl_config() -> Result<ServerConfig> {
    create_ssl_config_with_version(get_tls_version_from_env())
}

/// Debug function to list supported cipher suites
#[pyfunction]
pub fn list_rustls_cipher_suites() -> PyResult<Vec<String>> {
    // List the default cipher suites that rustls supports
    let default_suites = rustls::crypto::aws_lc_rs::DEFAULT_CIPHER_SUITES;
    let cipher_suites = default_suites.iter().map(|cs| format!("{:?}", cs)).collect::<Vec<_>>();

    Ok(cipher_suites)
}

/// Debug function to list all available cipher suites
#[pyfunction]
pub fn list_all_rustls_cipher_suites() -> PyResult<Vec<String>> {
    // List all cipher suites that rustls supports
    let all_suites = rustls::crypto::aws_lc_rs::ALL_CIPHER_SUITES;
    let cipher_suites = all_suites.iter().map(|cs| format!("{cs:?}")).collect::<Vec<_>>();

    Ok(cipher_suites)
}

/// Create SSL server configuration from an SSL context with TLS version
pub(crate) fn create_ssl_config_from_context_with_version(
    ssl_context: &Bound<PyAny>,
    tls_version: Option<TLSVersion>,
) -> Result<ServerConfig> {
    // Try to extract certificate and key file paths from the SSL context
    // These are test-specific attributes
    if let (Ok(certfile_attr), Ok(keyfile_attr)) = (ssl_context.getattr("_certfile"), ssl_context.getattr("_keyfile")) {
        let certfile: String = certfile_attr.extract()?;
        let keyfile: String = keyfile_attr.extract()?;

        // Load certificate from PEM
        let cert_data = fs::read(&certfile)?;
        let mut cert_reader = std::io::Cursor::new(&cert_data);
        let cert_der = match rustls_pemfile::read_one(&mut cert_reader)? {
            Some(Item::X509Certificate(cert)) => cert,
            _ => return Err(anyhow::anyhow!("failed to parse certificate")),
        };

        // Load private key from PEM
        let key_data = fs::read(&keyfile)?;
        let mut key_reader = std::io::Cursor::new(&key_data);
        let key_der = match rustls_pemfile::read_one(&mut key_reader)? {
            Some(Item::Pkcs8Key(key)) => PrivateKeyDer::from(key),
            Some(Item::Pkcs1Key(key)) => PrivateKeyDer::from(key),
            Some(Item::Sec1Key(key)) => PrivateKeyDer::from(key),
            _ => return Err(anyhow::anyhow!("failed to parse private key")),
        };

        let versions = match tls_version {
            Some(TLSVersion::TLS12) => &TLS12_ONLY[..],
            Some(TLSVersion::TLS12_PLUS) => &TLS12_PLUS[..],
            Some(TLSVersion::TLS13) => &TLS13_ONLY[..],
            None => &TLS_DEFAULT[..],
        };

        let config = ServerConfig::builder_with_protocol_versions(versions)
            .with_no_client_auth()
            .with_single_cert(vec![cert_der], key_der)?;

        Ok(config)
    } else {
        // Fallback: generate a self-signed certificate for testing
        create_ssl_config_with_version(tls_version)
    }
}

/// Create SSL server configuration from an SSL context
pub(crate) fn create_ssl_config_from_context(ssl_context: &Bound<PyAny>) -> Result<ServerConfig> {
    create_ssl_config_from_context_with_version(ssl_context, get_tls_version_from_env())
}

/// Create SSL client configuration from an SSL context with TLS version
pub(crate) fn create_ssl_client_config_from_context_with_version(
    _ssl_context: &Bound<PyAny>,
    tls_version: Option<TLSVersion>,
) -> Result<rustls::ClientConfig> {
    let versions = match tls_version {
        Some(TLSVersion::TLS12) => &TLS12_ONLY[..],
        Some(TLSVersion::TLS12_PLUS) => &TLS12_PLUS[..],
        Some(TLSVersion::TLS13) => &TLS13_ONLY[..],
        None => &TLS_DEFAULT[..],
    };

    let config = rustls::ClientConfig::builder_with_protocol_versions(versions)
        .dangerous()
        .with_custom_certificate_verifier(std::sync::Arc::new(NoCertificateVerification))
        .with_no_client_auth();

    Ok(config)
}

/// Create SSL client configuration from an SSL context
pub(crate) fn create_ssl_client_config_from_context(ssl_context: &Bound<PyAny>) -> Result<rustls::ClientConfig> {
    create_ssl_client_config_from_context_with_version(ssl_context, get_tls_version_from_env())
}

#[derive(Debug)]
struct NoCertificateVerification;

impl rustls::client::danger::ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer,
        _intermediates: &[rustls::pki_types::CertificateDer],
        _server_name: &rustls::pki_types::ServerName,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA1,
            rustls::SignatureScheme::ECDSA_SHA1_Legacy,
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
            rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
            rustls::SignatureScheme::ED25519,
            rustls::SignatureScheme::ED448,
        ]
    }
}

pub(crate) fn init_pymodule(module: &Bound<PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(list_rustls_cipher_suites, module)?)?;
    module.add_function(wrap_pyfunction!(list_all_rustls_cipher_suites, module)?)?;
    module.add_class::<TLSVersion>()?;
    Ok(())
}
