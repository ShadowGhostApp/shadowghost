use crate::core::types::Config;
use crate::crypto::types::*;
use crate::events::types::{AppEvent, CryptoEvent, EventBus};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::Arc;
use tokio::sync::RwLock;

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

pub struct SecurityManager {
    pub crypto: Arc<RwLock<CryptoManager>>,
    config: Config,
    event_bus: EventBus,
    trusted_keys: HashMap<String, PublicKey>,
    blocked_peers: HashMap<String, bool>,
}

impl SecurityManager {
    pub fn new(config: Config, event_bus: EventBus) -> Result<Self, String> {
        let crypto_manager =
            CryptoManager::new().map_err(|e| format!("Failed to create crypto manager: {}", e))?;

        Ok(Self {
            crypto: Arc::new(RwLock::new(crypto_manager)),
            config,
            event_bus,
            trusted_keys: HashMap::new(),
            blocked_peers: HashMap::new(),
        })
    }

    pub async fn initialize(&mut self) -> Result<(), String> {
        let crypto = self.crypto.read().await;
        let _public_key = crypto.get_public_key();

        self.event_bus
            .emit(AppEvent::Crypto(CryptoEvent::KeyPairGenerated));

        println!("Security manager initialized successfully");
        Ok(())
    }

    pub async fn encrypt_message(
        &self,
        message: &str,
        recipient_key: &PublicKey,
    ) -> Result<EncryptedMessage, String> {
        self.crypto
            .read()
            .await
            .encrypt_message(message, recipient_key)
            .map_err(|e| format!("Encryption failed: {}", e))
    }

    pub async fn decrypt_message(&self, encrypted: &EncryptedMessage) -> Result<String, String> {
        self.crypto
            .read()
            .await
            .decrypt_message(encrypted)
            .map_err(|e| format!("Decryption failed: {}", e))
    }

    pub async fn sign_data(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        self.crypto
            .read()
            .await
            .sign_data(data)
            .map_err(|e| format!("Signing failed: {}", e))
    }

    pub async fn verify_signature(
        &self,
        data: &[u8],
        signature: &[u8],
        public_key: &PublicKey,
    ) -> Result<bool, String> {
        self.crypto
            .read()
            .await
            .verify_signature(data, signature, public_key)
            .map_err(|e| format!("Signature verification failed: {}", e))
    }

    pub async fn get_public_key(&self) -> PublicKey {
        self.crypto.read().await.get_public_key()
    }

    pub fn add_trusted_key(&mut self, peer_id: String, public_key: PublicKey) {
        self.trusted_keys.insert(peer_id, public_key);
    }

    pub fn remove_trusted_key(&mut self, peer_id: &str) {
        self.trusted_keys.remove(peer_id);
    }

    pub fn is_peer_trusted(&self, peer_id: &str) -> bool {
        self.trusted_keys.contains_key(peer_id)
    }

    pub fn block_peer(&mut self, peer_id: String) {
        self.blocked_peers.insert(peer_id, true);
    }

    pub fn unblock_peer(&mut self, peer_id: &str) {
        self.blocked_peers.remove(peer_id);
    }

    pub fn is_peer_blocked(&self, peer_id: &str) -> bool {
        self.blocked_peers.get(peer_id).copied().unwrap_or(false)
    }

    pub fn get_trusted_keys(&self) -> &HashMap<String, PublicKey> {
        &self.trusted_keys
    }

    pub fn get_blocked_peers(&self) -> Vec<String> {
        self.blocked_peers.keys().cloned().collect()
    }

    pub async fn derive_shared_secret(
        &self,
        other_public_key: &PublicKey,
    ) -> Result<Vec<u8>, String> {
        self.crypto
            .read()
            .await
            .derive_shared_secret(other_public_key)
            .map_err(|e| format!("Shared secret derivation failed: {}", e))
    }

    pub async fn hash_data(&self, data: &[u8]) -> Vec<u8> {
        self.crypto.read().await.hash_data(data)
    }

    pub fn validate_peer_identity(&self, peer_id: &str, public_key: &PublicKey) -> bool {
        // Simple validation - check if we have this peer's key and it matches
        if let Some(stored_key) = self.trusted_keys.get(peer_id) {
            stored_key.key_data == public_key.key_data
        } else {
            false
        }
    }

    pub fn get_security_level(&self, peer_id: &str) -> SecurityLevel {
        if self.is_peer_blocked(peer_id) {
            SecurityLevel::blocked()
        } else if self.is_peer_trusted(peer_id) {
            SecurityLevel::trusted()
        } else {
            SecurityLevel::unknown()
        }
    }

    pub async fn encrypt_for_storage(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        self.crypto
            .read()
            .await
            .encrypt(data)
            .map_err(|e| format!("Storage encryption failed: {}", e))
    }

    pub async fn decrypt_from_storage(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, String> {
        self.crypto
            .read()
            .await
            .decrypt(encrypted_data)
            .map_err(|e| format!("Storage decryption failed: {}", e))
    }

    pub fn clear_all_trusted_keys(&mut self) {
        self.trusted_keys.clear();
    }

    pub fn clear_all_blocked_peers(&mut self) {
        self.blocked_peers.clear();
    }

    pub fn get_trust_stats(&self) -> TrustStats {
        TrustStats {
            trusted_peers: self.trusted_keys.len(),
            blocked_peers: self.blocked_peers.len(),
            total_known_peers: self.trusted_keys.len() + self.blocked_peers.len(),
        }
    }

    pub fn export_trusted_keys(&self) -> Result<String, String> {
        serde_json::to_string_pretty(&self.trusted_keys)
            .map_err(|e| format!("Failed to export trusted keys: {}", e))
    }

    pub fn import_trusted_keys(&mut self, keys_data: &str) -> Result<usize, String> {
        let imported_keys: HashMap<String, PublicKey> = serde_json::from_str(keys_data)
            .map_err(|e| format!("Failed to import trusted keys: {}", e))?;

        let mut imported_count = 0;
        for (peer_id, key) in imported_keys {
            if !self.trusted_keys.contains_key(&peer_id) {
                self.trusted_keys.insert(peer_id, key);
                imported_count += 1;
            }
        }

        Ok(imported_count)
    }
}