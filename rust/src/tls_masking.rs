use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum TlsError {
    HandshakeFailed(String),
    CertificateError(String),
    ConnectionError(String),
}

impl fmt::Display for TlsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TlsError::HandshakeFailed(msg) => write!(f, "TLS handshake failed: {}", msg),
            TlsError::CertificateError(msg) => write!(f, "Certificate error: {}", msg),
            TlsError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
        }
    }
}

impl Error for TlsError {}

pub struct TlsMasking {
    enabled: bool,
    fake_certificates: Vec<Vec<u8>>,
}

impl TlsMasking {
    pub fn new() -> Self {
        Self {
            enabled: false,
            fake_certificates: vec![],
        }
    }

    pub fn enable(&mut self) -> Result<(), TlsError> {
        self.enabled = true;
        self.generate_fake_certificates()?;
        Ok(())
    }

    pub fn disable(&mut self) {
        self.enabled = false;
        self.fake_certificates.clear();
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub async fn wrap_connection(&self, data: &[u8]) -> Result<Vec<u8>, TlsError> {
        if !self.enabled {
            return Ok(data.to_vec());
        }

        let mut wrapped = Vec::new();
        wrapped.extend_from_slice(&self.create_tls_header());
        wrapped.extend_from_slice(data);
        wrapped.extend_from_slice(&self.create_tls_footer());

        Ok(wrapped)
    }

    pub async fn unwrap_connection(&self, data: &[u8]) -> Result<Vec<u8>, TlsError> {
        if !self.enabled {
            return Ok(data.to_vec());
        }

        if data.len() < 20 {
            return Err(TlsError::ConnectionError("Data too short".to_string()));
        }

        let header_size = 10;
        let footer_size = 10;

        if data.len() < header_size + footer_size {
            return Err(TlsError::ConnectionError("Invalid TLS frame".to_string()));
        }

        let payload = &data[header_size..data.len() - footer_size];
        Ok(payload.to_vec())
    }

    fn generate_fake_certificates(&mut self) -> Result<(), TlsError> {
        let fake_cert = vec![
            0x30, 0x82, 0x01, 0x00, 0x30, 0x81, 0xB2, 0x02, 0x01, 0x01, 0x30, 0x0D, 0x06, 0x09,
            0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x01, 0x0B, 0x05, 0x00, 0x30, 0x1A, 0x31,
            0x18, 0x30, 0x16, 0x06, 0x03, 0x55, 0x04, 0x03, 0x0C, 0x0F, 0x65, 0x78, 0x61, 0x6D,
            0x70, 0x6C, 0x65, 0x2E, 0x63, 0x6F, 0x6D,
        ];

        self.fake_certificates.push(fake_cert);
        Ok(())
    }

    fn create_tls_header(&self) -> Vec<u8> {
        vec![0x16, 0x03, 0x03, 0x00, 0x05, 0x01, 0x00, 0x00, 0x01, 0x01]
    }

    fn create_tls_footer(&self) -> Vec<u8> {
        vec![0x17, 0x03, 0x03, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x01]
    }

    pub fn create_fake_handshake(&self) -> Vec<u8> {
        let mut handshake = Vec::new();
        handshake.extend_from_slice(&[0x16, 0x03, 0x03, 0x00, 0x86]);
        handshake.extend_from_slice(&[0x01, 0x00, 0x00, 0x82]);
        handshake.extend_from_slice(&[0x03, 0x03]);

        let mut random = vec![0u8; 32];
        for i in 0..32 {
            random[i] = (i as u8) ^ 0xAA;
        }
        handshake.extend_from_slice(&random);

        handshake.extend_from_slice(&[0x00]);
        handshake.extend_from_slice(&[0x00, 0x08]);
        handshake.extend_from_slice(&[0x00, 0x2F, 0x00, 0x35, 0x00, 0x0A, 0x00, 0x39]);
        handshake.extend_from_slice(&[0x01, 0x00]);

        handshake
    }

    pub fn validate_tls_frame(&self, data: &[u8]) -> bool {
        if data.len() < 5 {
            return false;
        }

        let content_type = data[0];
        let version = u16::from_be_bytes([data[1], data[2]]);
        let length = u16::from_be_bytes([data[3], data[4]]) as usize;

        content_type >= 0x14
            && content_type <= 0x18
            && (version == 0x0301 || version == 0x0302 || version == 0x0303)
            && data.len() >= 5 + length
    }
}
