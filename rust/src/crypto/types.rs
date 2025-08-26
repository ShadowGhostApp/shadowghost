use serde::{Deserialize, Serialize};

// Re-export from crypto module for convenience
pub use crate::crypto::{CryptoError, EncryptedMessage, KeyPair, PrivateKey, PublicKey};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityLevel {
    pub level: SecurityLevelType,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLevelType {
    Trusted,
    Unknown,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustStats {
    pub trusted_peers: usize,
    pub blocked_peers: usize,
    pub total_known_peers: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoStats {
    pub messages_encrypted: u64,
    pub messages_decrypted: u64,
    pub signatures_created: u64,
    pub signatures_verified: u64,
    pub key_exchanges: u64,
    pub encryption_errors: u32,
    pub decryption_errors: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyInfo {
    pub algorithm: String,
    pub key_size: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateInfo {
    pub subject: String,
    pub issuer: String,
    pub valid_from: chrono::DateTime<chrono::Utc>,
    pub valid_to: chrono::DateTime<chrono::Utc>,
    pub fingerprint: String,
    pub is_valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionSettings {
    pub default_algorithm: String,
    pub key_size: u32,
    pub signature_algorithm: String,
    pub hash_algorithm: String,
}

impl Default for EncryptionSettings {
    fn default() -> Self {
        Self {
            default_algorithm: "ChaCha20Poly1305".to_string(),
            key_size: 256,
            signature_algorithm: "Ed25519".to_string(),
            hash_algorithm: "SHA256".to_string(),
        }
    }
}

impl Default for CryptoStats {
    fn default() -> Self {
        Self {
            messages_encrypted: 0,
            messages_decrypted: 0,
            signatures_created: 0,
            signatures_verified: 0,
            key_exchanges: 0,
            encryption_errors: 0,
            decryption_errors: 0,
        }
    }
}

impl SecurityLevel {
    pub fn trusted() -> Self {
        Self {
            level: SecurityLevelType::Trusted,
            description: "This peer is trusted and verified".to_string(),
        }
    }

    pub fn unknown() -> Self {
        Self {
            level: SecurityLevelType::Unknown,
            description: "This peer's identity has not been verified".to_string(),
        }
    }

    pub fn blocked() -> Self {
        Self {
            level: SecurityLevelType::Blocked,
            description: "This peer has been blocked and will be rejected".to_string(),
        }
    }
}
