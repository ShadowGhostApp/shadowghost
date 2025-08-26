use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum CryptoError {
    EncryptionFailed(String),
    DecryptionFailed(String),
    KeyGenerationFailed(String),
    InvalidKey(String),
    SigningFailed(String),
    VerificationFailed(String),
}

impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CryptoError::EncryptionFailed(msg) => write!(f, "Encryption failed: {}", msg),
            CryptoError::DecryptionFailed(msg) => write!(f, "Decryption failed: {}", msg),
            CryptoError::KeyGenerationFailed(msg) => write!(f, "Key generation failed: {}", msg),
            CryptoError::InvalidKey(msg) => write!(f, "Invalid key: {}", msg),
            CryptoError::SigningFailed(msg) => write!(f, "Signing failed: {}", msg),
            CryptoError::VerificationFailed(msg) => write!(f, "Verification failed: {}", msg),
        }
    }
}

impl Error for CryptoError {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
pub struct PrivateKey {
    pub key_data: Vec<u8>,
    pub algorithm: String,
}

impl PrivateKey {
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
    pub algorithm: String,
}

impl EncryptedMessage {
    pub fn new(data: Vec<u8>, nonce: Vec<u8>) -> Self {
        Self {
            data,
            nonce,
            algorithm: "ChaCha20Poly1305".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPair {
    pub public_key: PublicKey,
    pub private_key: PrivateKey,
}

pub struct CryptoManager {
    keypair: Option<KeyPair>,
}

impl CryptoManager {
    pub fn new() -> Result<Self, CryptoError> {
        let mut manager = Self { keypair: None };
        manager.generate_keypair()?;
        Ok(manager)
    }

    pub fn generate_keypair(&mut self) -> Result<(), CryptoError> {
        // Для демонстрации создаем фиктивные ключи
        // В реальной реализации здесь должна быть настоящая криптография
        let private_key_data = (0..32).collect::<Vec<u8>>();
        let public_key_data = (32..64).collect::<Vec<u8>>();

        let keypair = KeyPair {
            private_key: PrivateKey::new(private_key_data),
            public_key: PublicKey::new(public_key_data),
        };

        self.keypair = Some(keypair);
        Ok(())
    }

    pub fn get_public_key(&self) -> PublicKey {
        self.keypair
            .as_ref()
            .map(|kp| kp.public_key.clone())
            .unwrap_or_else(|| PublicKey::new(vec![]))
    }

    pub fn encrypt_message(
        &self,
        message: &str,
        _recipient_key: &PublicKey,
    ) -> Result<EncryptedMessage, CryptoError> {
        // Фиктивное шифрование для демонстрации
        let data = message.as_bytes().to_vec();
        let nonce = (0..12).collect::<Vec<u8>>();

        Ok(EncryptedMessage::new(data, nonce))
    }

    pub fn decrypt_message(&self, encrypted: &EncryptedMessage) -> Result<String, CryptoError> {
        // Фиктивная расшифровка для демонстрации
        String::from_utf8(encrypted.data.clone())
            .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))
    }

    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        // Фиктивное шифрование для хранения
        Ok(data.to_vec())
    }

    pub fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        // Фиктивная расшифровка для хранения
        Ok(encrypted_data.to_vec())
    }

    pub fn sign_data(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        // Фиктивная подпись
        let signature = data.iter().map(|b| b.wrapping_add(1)).collect();
        Ok(signature)
    }

    pub fn verify_signature(
        &self,
        _data: &[u8],
        _signature: &[u8],
        _public_key: &PublicKey,
    ) -> Result<bool, CryptoError> {
        // Фиктивная проверка подписи
        Ok(true)
    }

    pub fn derive_shared_secret(&self, _other_key: &PublicKey) -> Result<Vec<u8>, CryptoError> {
        // Фиктивный общий секрет
        Ok((0..32).collect::<Vec<u8>>())
    }

    pub fn hash_data(&self, data: &[u8]) -> Vec<u8> {
        // Простое хеширование для демонстрации
        let mut hash = vec![0u8; 32];
        for (i, byte) in data.iter().enumerate() {
            hash[i % 32] ^= byte;
        }
        hash
    }

    pub fn export_public_key(&self) -> Result<String, CryptoError> {
        let public_key = self.get_public_key();
        serde_json::to_string(&public_key).map_err(|e| CryptoError::InvalidKey(e.to_string()))
    }

    pub fn import_public_key(&self, key_data: &str) -> Result<PublicKey, CryptoError> {
        serde_json::from_str(key_data).map_err(|e| CryptoError::InvalidKey(e.to_string()))
    }
}

impl Default for CryptoManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self { keypair: None })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_manager_creation() {
        let manager = CryptoManager::new().unwrap();
        assert!(manager.keypair.is_some());
    }

    #[test]
    fn test_encrypt_decrypt() {
        let manager = CryptoManager::new().unwrap();
        let message = "Hello, World!";
        let public_key = manager.get_public_key();

        let encrypted = manager.encrypt_message(message, &public_key).unwrap();
        let decrypted = manager.decrypt_message(&encrypted).unwrap();

        assert_eq!(message, decrypted);
    }

    #[test]
    fn test_sign_verify() {
        let manager = CryptoManager::new().unwrap();
        let data = b"test data";
        let public_key = manager.get_public_key();

        let signature = manager.sign_data(data).unwrap();
        let is_valid = manager
            .verify_signature(data, &signature, &public_key)
            .unwrap();

        assert!(is_valid);
    }
}
