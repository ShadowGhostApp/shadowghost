use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum CryptoError {
    KeyGeneration(String),
    Encryption(String),
    Decryption(String),
    InvalidKey(String),
}

impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CryptoError::KeyGeneration(msg) => write!(f, "Key generation error: {}", msg),
            CryptoError::Encryption(msg) => write!(f, "Encryption error: {}", msg),
            CryptoError::Decryption(msg) => write!(f, "Decryption error: {}", msg),
            CryptoError::InvalidKey(msg) => write!(f, "Invalid key: {}", msg),
        }
    }
}

impl Error for CryptoError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKey {
    pub key_data: Vec<u8>,
    pub algorithm: String,
}

impl PublicKey {
    pub fn new(key_data: Vec<u8>) -> Self {
        Self {
            key_data,
            algorithm: "Ed25519".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMessage {
    pub data: Vec<u8>,
    pub nonce: Vec<u8>,
    pub sender_key: PublicKey,
}

pub struct CryptoManager {
    _private_key: Vec<u8>,
    public_key: PublicKey,
}

impl CryptoManager {
    pub fn new() -> Result<Self, CryptoError> {
        let private_key = vec![0u8; 32];
        let public_key = PublicKey::new(vec![0u8; 32]);

        Ok(Self {
            _private_key: private_key,
            public_key,
        })
    }

    pub fn get_public_key(&self) -> PublicKey {
        self.public_key.clone()
    }

    pub fn encrypt_message(
        &self,
        message: &str,
        _recipient_key: &PublicKey,
    ) -> Result<EncryptedMessage, CryptoError> {
        let data = message.as_bytes().to_vec();
        let nonce = vec![0u8; 24];

        Ok(EncryptedMessage {
            data,
            nonce,
            sender_key: self.public_key.clone(),
        })
    }

    pub fn decrypt_message(&self, encrypted: &EncryptedMessage) -> Result<String, CryptoError> {
        String::from_utf8(encrypted.data.clone())
            .map_err(|e| CryptoError::Decryption(e.to_string()))
    }

    pub fn sign_data(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        Ok(data.to_vec())
    }

    pub fn verify_signature(
        &self,
        data: &[u8],
        signature: &[u8],
        _public_key: &PublicKey,
    ) -> Result<bool, CryptoError> {
        Ok(data == signature)
    }

    pub fn generate_keypair() -> Result<(Vec<u8>, PublicKey), CryptoError> {
        let private_key = vec![0u8; 32];
        let public_key = PublicKey::new(vec![0u8; 32]);
        Ok((private_key, public_key))
    }
}
