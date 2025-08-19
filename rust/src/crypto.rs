use curve25519_dalek::{constants::X25519_BASEPOINT, scalar::Scalar};
use ring::rand::SecureRandom;
use ring::{digest, rand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKey {
    pub key_bytes: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMessage {
    pub data: Vec<u8>,
    pub nonce: Vec<u8>,
    pub auth_tag: Vec<u8>,
}

pub struct CryptoManager {
    private_key: Vec<u8>,
    public_key: PublicKey,
    peer_keys: HashMap<String, PublicKey>,
    rng: rand::SystemRandom,
}

impl CryptoManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let rng = rand::SystemRandom::new();
        let private_key_bytes = Self::generate_private_key(&rng)?;
        let public_key_bytes = Self::derive_public_key(&private_key_bytes)?;

        let public_key = PublicKey {
            key_bytes: public_key_bytes,
        };

        Ok(Self {
            private_key: private_key_bytes,
            public_key,
            peer_keys: HashMap::new(),
            rng,
        })
    }

    fn generate_private_key(
        rng: &rand::SystemRandom,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut private_key = [0u8; 32];
        rng.fill(&mut private_key)
            .map_err(|_| "Failed to generate private key")?;
        Ok(private_key.to_vec())
    }

    fn derive_public_key(private_key: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        if private_key.len() != 32 {
            return Err("Private key must be 32 bytes".into());
        }

        let private_array: [u8; 32] = private_key
            .try_into()
            .map_err(|_| "Invalid private key length")?;

        let scalar = Scalar::from_bytes_mod_order(private_array);
        let point = &X25519_BASEPOINT * &scalar;

        Ok(point.to_bytes().to_vec())
    }

    pub fn hash_data(&self, data: &[u8]) -> Vec<u8> {
        let digest = digest::digest(&digest::SHA256, data);
        digest.as_ref().to_vec()
    }

    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        use ring::aead;

        let key_bytes = &self.private_key[..32];
        let unbound_key = aead::UnboundKey::new(&aead::CHACHA20_POLY1305, key_bytes)
            .map_err(|_| "Failed to create encryption key")?;
        let key = aead::LessSafeKey::new(unbound_key);

        let mut nonce_bytes = [0u8; 12];
        self.rng
            .fill(&mut nonce_bytes)
            .map_err(|_| "Failed to generate nonce")?;
        let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes);

        let mut in_out = data.to_vec();
        key.seal_in_place_append_tag(nonce, aead::Aad::empty(), &mut in_out)
            .map_err(|_| "Encryption failed")?;

        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&in_out);
        Ok(result)
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        if data.len() < 12 {
            return Err("Invalid encrypted data".into());
        }

        use ring::aead;

        let key_bytes = &self.private_key[..32];
        let unbound_key = aead::UnboundKey::new(&aead::CHACHA20_POLY1305, key_bytes)
            .map_err(|_| "Failed to create decryption key")?;
        let key = aead::LessSafeKey::new(unbound_key);

        let nonce_bytes: [u8; 12] = data[..12].try_into().map_err(|_| "Invalid nonce")?;
        let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes);

        let mut in_out = data[12..].to_vec();
        let decrypted = key
            .open_in_place(nonce, aead::Aad::empty(), &mut in_out)
            .map_err(|_| "Decryption failed")?;

        Ok(decrypted.to_vec())
    }

    pub fn get_public_key(&self) -> PublicKey {
        self.public_key.clone()
    }

    pub fn sign_data(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let hash = self.hash_data(data);
        let mut signature = Vec::new();
        signature.extend_from_slice(&self.private_key[..32]);
        signature.extend_from_slice(&hash);
        Ok(signature)
    }

    pub fn verify_signature(
        &self,
        data: &[u8],
        signature: &[u8],
        public_key: &PublicKey,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        if signature.len() < 64 {
            return Ok(false);
        }

        let expected_hash = self.hash_data(data);
        let signature_hash = &signature[32..64];

        Ok(expected_hash == signature_hash && signature[..32] == public_key.key_bytes[..32])
    }

    pub fn encrypt_message(
        &self,
        data: &[u8],
        _peer_id: &str,
    ) -> Result<EncryptedMessage, Box<dyn std::error::Error>> {
        let encrypted = self.encrypt(data)?;

        Ok(EncryptedMessage {
            data: encrypted[12..].to_vec(),
            nonce: encrypted[..12].to_vec(),
            auth_tag: Vec::new(),
        })
    }

    pub fn decrypt_message(
        &self,
        encrypted_msg: &EncryptedMessage,
        _peer_id: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut combined = encrypted_msg.nonce.clone();
        combined.extend_from_slice(&encrypted_msg.data);
        self.decrypt(&combined)
    }

    pub fn add_peer_key(&mut self, peer_id: String, public_key: PublicKey) {
        self.peer_keys.insert(peer_id, public_key);
    }

    pub fn get_peer_key(&self, peer_id: &str) -> Option<&PublicKey> {
        self.peer_keys.get(peer_id)
    }

    pub fn derive_shared_secret(
        &self,
        peer_public_key: &PublicKey,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        if peer_public_key.key_bytes.len() != 32 {
            return Err("Invalid peer public key length".into());
        }

        if self.private_key.len() != 32 {
            return Err("Invalid private key length".into());
        }

        let private_array: [u8; 32] = self.private_key[..32]
            .try_into()
            .map_err(|_| "Invalid private key length")?;

        let peer_public_array: [u8; 32] = peer_public_key.key_bytes[..32]
            .try_into()
            .map_err(|_| "Invalid peer public key length")?;

        let my_scalar = Scalar::from_bytes_mod_order(private_array);
        let peer_point = curve25519_dalek::montgomery::MontgomeryPoint(peer_public_array);

        let shared_point = my_scalar * peer_point;
        Ok(shared_point.to_bytes().to_vec())
    }
}
