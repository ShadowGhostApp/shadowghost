use crate::core::config::AppConfig;
use crate::events::EventBus;
use crate::security::crypto::{CryptoManager, EncryptedMessage, PublicKey};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct SecurityManager {
    pub crypto: Arc<RwLock<CryptoManager>>,
    config: AppConfig,
    event_bus: EventBus,
    trusted_keys: HashMap<String, PublicKey>,
    blocked_peers: HashMap<String, bool>,
}

impl SecurityManager {
    pub fn new(config: AppConfig, event_bus: EventBus) -> Result<Self, String> {
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
        // Generate keypair if needed
        let crypto = self.crypto.read().await;
        let _public_key = crypto.get_public_key();

        // Emit crypto initialization event
        use crate::events::{AppEvent, CryptoEvent};
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
            SecurityLevel::Blocked
        } else if self.is_peer_trusted(peer_id) {
            SecurityLevel::Trusted
        } else {
            SecurityLevel::Unknown
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

#[derive(Debug, Clone)]
pub enum SecurityLevel {
    Trusted,
    Unknown,
    Blocked,
}

#[derive(Debug, Clone)]
pub struct TrustStats {
    pub trusted_peers: usize,
    pub blocked_peers: usize,
    pub total_known_peers: usize,
}
