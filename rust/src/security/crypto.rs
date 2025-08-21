use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum CryptoError {
    KeyGeneration(String),
    Encryption(String),
    Decryption(String),
    InvalidKey(String),
    SignatureError(String),
}

impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CryptoError::KeyGeneration(msg) => write!(f, "Key generation error: {}", msg),
            CryptoError::Encryption(msg) => write!(f, "Encryption error: {}", msg),
            CryptoError::Decryption(msg) => write!(f, "Decryption error: {}", msg),
            CryptoError::InvalidKey(msg) => write!(f, "Invalid key: {}", msg),
            CryptoError::SignatureError(msg) => write!(f, "Signature error: {}", msg),
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
    private_key: Vec<u8>,
    public_key: PublicKey,
}

impl CryptoManager {
    pub fn new() -> Result<Self, CryptoError> {
        let private_key = Self::generate_private_key()?;
        let public_key_data = Self::derive_public_key(&private_key)?;
        let public_key = PublicKey::new(public_key_data);

        Ok(Self {
            private_key,
            public_key,
        })
    }

    pub fn get_public_key(&self) -> PublicKey {
        self.public_key.clone()
    }

    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        // Simple XOR encryption for demonstration purposes
        let mut encrypted = data.to_vec();
        for (i, byte) in encrypted.iter_mut().enumerate() {
            *byte ^= self.private_key[i % self.private_key.len()];
        }
        Ok(encrypted)
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        // Simple XOR decryption for demonstration purposes
        let mut decrypted = data.to_vec();
        for (i, byte) in decrypted.iter_mut().enumerate() {
            *byte ^= self.private_key[i % self.private_key.len()];
        }
        Ok(decrypted)
    }

    pub fn encrypt_message(
        &self,
        message: &str,
        _recipient_key: &PublicKey,
    ) -> Result<EncryptedMessage, CryptoError> {
        let data = self.encrypt(message.as_bytes())?;
        let nonce = vec![0u8; 24];

        Ok(EncryptedMessage {
            data,
            nonce,
            sender_key: self.public_key.clone(),
        })
    }

    pub fn decrypt_message(&self, encrypted: &EncryptedMessage) -> Result<String, CryptoError> {
        let decrypted = self.decrypt(&encrypted.data)?;
        String::from_utf8(decrypted).map_err(|e| CryptoError::Decryption(e.to_string()))
    }

    pub fn sign_data(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        // Simple signature for demonstration purposes
        let mut signature = Vec::new();
        let hash = self.hash_data(data);
        signature.extend_from_slice(&hash);
        signature.extend_from_slice(&self.private_key[..16]); // First 16 bytes of private key
        Ok(signature)
    }

    pub fn verify_signature(
        &self,
        data: &[u8],
        signature: &[u8],
        _public_key: &PublicKey,
    ) -> Result<bool, CryptoError> {
        if signature.len() < 48 {
            // 32 (hash) + 16 (key part)
            return Ok(false);
        }

        let data_hash = self.hash_data(data);
        let sig_hash = &signature[..32];

        Ok(data_hash == sig_hash)
    }

    pub fn derive_shared_secret(
        &self,
        other_public_key: &PublicKey,
    ) -> Result<Vec<u8>, CryptoError> {
        // Simple shared secret derivation for demonstration purposes
        let mut secret = Vec::new();
        for i in 0..32 {
            let a = self.private_key[i % self.private_key.len()];
            let b = other_public_key.key_data[i % other_public_key.key_data.len()];
            secret.push(a ^ b);
        }
        Ok(secret)
    }

    pub fn hash_data(&self, data: &[u8]) -> Vec<u8> {
        // Simple hash function for demonstration purposes
        let mut hash = vec![0u8; 32];
        for (i, &byte) in data.iter().enumerate() {
            hash[i % 32] ^= byte;
        }

        for i in 0..32 {
            hash[i] = hash[i].wrapping_add(i as u8);
        }
        hash
    }

    pub fn generate_keypair() -> Result<(Vec<u8>, PublicKey), CryptoError> {
        let private_key = Self::generate_private_key()?;
        let public_key_data = Self::derive_public_key(&private_key)?;
        let public_key = PublicKey::new(public_key_data);
        Ok((private_key, public_key))
    }

    fn generate_private_key() -> Result<Vec<u8>, CryptoError> {
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| CryptoError::KeyGeneration(e.to_string()))?
            .as_nanos();

        let mut key = Vec::new();
        let mut seed = timestamp;

        for _ in 0..32 {
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            key.push((seed >> 16) as u8);
        }

        Ok(key)
    }

    fn derive_public_key(private_key: &[u8]) -> Result<Vec<u8>, CryptoError> {
        // Simple public key derivation for demonstration purposes
        let mut public_key = vec![0u8; 32];
        for i in 0..32 {
            public_key[i] = private_key[i].wrapping_mul(7).wrapping_add(13);
        }
        Ok(public_key)
    }
}
