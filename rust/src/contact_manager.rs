use crate::crypto::CryptoManager;
use crate::events::EventBus;
use crate::network::{Contact, ContactStatus, PeerData, TrustLevel};
use crate::peer::Peer;
use base64::{engine::general_purpose, Engine as _};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug)]
pub enum ContactError {
    InvalidLinkFormat(String),
    DecodeError(String),
    ParseError(String),
    DuplicateContact(String),
    NotFound(String),
    Crypto(String),
}

impl std::fmt::Display for ContactError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContactError::InvalidLinkFormat(msg) => write!(f, "Invalid link format: {}", msg),
            ContactError::DecodeError(msg) => write!(f, "Decode error: {}", msg),
            ContactError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ContactError::DuplicateContact(msg) => write!(f, "Duplicate contact: {}", msg),
            ContactError::NotFound(msg) => write!(f, "Contact not found: {}", msg),
            ContactError::Crypto(msg) => write!(f, "Crypto error: {}", msg),
        }
    }
}

impl std::error::Error for ContactError {}

pub struct ContactManager {
    local_peer: Peer,
    contacts: Arc<RwLock<HashMap<String, Contact>>>,
    crypto: Arc<RwLock<CryptoManager>>,
    event_bus: EventBus,
}

impl ContactManager {
    pub fn new(local_peer: Peer, crypto: Arc<RwLock<CryptoManager>>, event_bus: EventBus) -> Self {
        Self {
            local_peer,
            contacts: Arc::new(RwLock::new(HashMap::new())),
            crypto,
            event_bus,
        }
    }

    pub async fn generate_sg_link(&self) -> Result<String, ContactError> {
        log::info!("Generating SG link for peer: {}", self.local_peer.name);

        let peer_data = PeerData {
            id: self.local_peer.id.clone(),
            name: self.local_peer.name.clone(),
            address: self.local_peer.address.clone(),
            public_key: {
                let crypto = self.crypto.read().await;
                crypto.get_public_key().key_bytes
            },
            connected_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| ContactError::Crypto(e.to_string()))?
                .as_secs(),
        };

        let data = serde_json::to_string(&peer_data)
            .map_err(|e| ContactError::ParseError(e.to_string()))?;

        log::debug!("JSON data to encode: {}", data);
        log::debug!("JSON data bytes: {:?}", data.as_bytes());

        let encoded = general_purpose::URL_SAFE_NO_PAD.encode(data.as_bytes());
        log::debug!("Generated base64: {}", encoded);

        // Тестируем декодирование сразу после генерации
        match general_purpose::URL_SAFE_NO_PAD.decode(&encoded) {
            Ok(test_decoded) => match String::from_utf8(test_decoded) {
                Ok(test_json) => {
                    log::debug!("Self-test successful - decoded back to: {}", test_json);
                    if test_json != data {
                        log::error!("Self-test failed - JSON doesn't match!");
                        return Err(ContactError::ParseError(
                            "Link generation self-test failed".to_string(),
                        ));
                    }
                }
                Err(e) => {
                    log::error!("Self-test failed - UTF-8 error: {}", e);
                    return Err(ContactError::ParseError(
                        "Link generation produces invalid UTF-8".to_string(),
                    ));
                }
            },
            Err(e) => {
                log::error!("Self-test failed - base64 decode error: {}", e);
                return Err(ContactError::ParseError(
                    "Link generation produces invalid base64".to_string(),
                ));
            }
        }

        let sg_link = format!("sg://{}", encoded);
        log::debug!("Final SG link: {}", sg_link);

        Ok(sg_link)
    }

    fn normalize_base64(input: &str) -> String {
        let mut normalized = input
            .chars()
            .map(|c| match c {
                '-' => '+',
                '_' => '/',
                other => other,
            })
            .collect::<String>();

        while normalized.len() % 4 != 0 {
            normalized.push('=');
        }

        normalized
    }

    pub async fn add_contact_by_sg_link(&self, sg_link: &str) -> Result<Contact, ContactError> {
        log::info!("Adding contact by SG link");
        log::debug!("Full SG link: '{}'", sg_link);

        if !sg_link.starts_with("sg://") {
            return Err(ContactError::InvalidLinkFormat(
                "Link must start with sg://".to_string(),
            ));
        }

        let encoded = &sg_link[5..];
        log::debug!("Extracted encoded part: '{}'", encoded);
        log::debug!("Encoded length: {}", encoded.len());

        // Проверяем, что encoded часть содержит только валидные base64 символы
        let valid_chars = encoded.chars().all(|c| {
            c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=' || c == '-' || c == '_'
        });

        if !valid_chars {
            log::error!("Invalid characters in base64 string");
            return Err(ContactError::InvalidLinkFormat(
                "Contains invalid base64 characters".to_string(),
            ));
        }

        let decoded_bytes = self.try_decode_base64(encoded)?;
        log::debug!(
            "Decoded {} bytes: {:?}",
            decoded_bytes.len(),
            &decoded_bytes[..std::cmp::min(decoded_bytes.len(), 20)]
        );

        // Проверяем, начинается ли декодированная строка с '{' (JSON)
        if decoded_bytes.is_empty() || decoded_bytes[0] != b'{' {
            log::error!("Decoded data doesn't start with JSON object");
            return Err(ContactError::DecodeError(
                "Data is not JSON format".to_string(),
            ));
        }

        // Проверяем, что все байты являются валидными UTF-8 символами
        match std::str::from_utf8(&decoded_bytes) {
            Ok(data_str) => {
                log::debug!("Successfully converted to UTF-8: {}", data_str);

                let peer_data: PeerData = serde_json::from_str(data_str)
                    .map_err(|e| ContactError::ParseError(format!("JSON parse failed: {}", e)))?;

                if peer_data.id == self.local_peer.id {
                    return Err(ContactError::DuplicateContact(
                        "Cannot add yourself as contact".to_string(),
                    ));
                }

                log::info!("Successfully parsed peer data for: {}", peer_data.name);

                let contact = Contact {
                    id: peer_data.id.clone(),
                    name: peer_data.name.clone(),
                    address: peer_data.address,
                    status: ContactStatus::Offline,
                    trust_level: TrustLevel::Low,
                    last_seen: peer_data.connected_at,
                };

                {
                    let mut contacts = self.contacts.write().await;
                    if contacts.contains_key(&peer_data.id) {
                        log::warn!("Contact '{}' already exists, updating", contact.name);
                    }
                    contacts.insert(peer_data.id.clone(), contact.clone());
                }

                {
                    let mut crypto = self.crypto.write().await;
                    crypto.add_peer_key(
                        peer_data.id,
                        crate::crypto::PublicKey {
                            key_bytes: peer_data.public_key,
                        },
                    );
                }

                log::info!("Contact '{}' added successfully", contact.name);

                self.event_bus
                    .emit_network(crate::events::NetworkEvent::ContactAdded {
                        contact: contact.clone(),
                    });

                Ok(contact)
            }
            Err(utf8_error) => {
                log::error!("UTF-8 conversion failed: {}", utf8_error);
                log::error!("Invalid UTF-8 bytes: {:?}", &decoded_bytes);

                // Пытаемся найти где именно проблема
                for (i, &byte) in decoded_bytes.iter().enumerate() {
                    if byte > 127 {
                        log::error!("Non-ASCII byte at position {}: 0x{:02x}", i, byte);
                    }
                }

                Err(ContactError::DecodeError(format!(
                    "UTF-8 conversion failed: {}. Decoded {} bytes, invalid sequence at position {}",
                    utf8_error, decoded_bytes.len(), utf8_error.valid_up_to()
                )))
            }
        }
    }

    fn try_decode_base64(&self, encoded: &str) -> Result<Vec<u8>, ContactError> {
        log::debug!("Trying to decode base64: '{}'", encoded);

        let decode_attempts: Vec<Box<dyn Fn() -> Result<Vec<u8>, base64::DecodeError>>> = vec![
            Box::new(|| {
                log::debug!("Trying URL_SAFE_NO_PAD");
                general_purpose::URL_SAFE_NO_PAD.decode(encoded)
            }),
            Box::new(|| {
                log::debug!("Trying URL_SAFE");
                general_purpose::URL_SAFE.decode(encoded)
            }),
            Box::new(|| {
                log::debug!("Trying STANDARD_NO_PAD");
                general_purpose::STANDARD_NO_PAD.decode(encoded)
            }),
            Box::new(|| {
                log::debug!("Trying STANDARD");
                general_purpose::STANDARD.decode(encoded)
            }),
            Box::new(|| {
                log::debug!("Trying normalized STANDARD");
                let normalized = Self::normalize_base64(encoded);
                log::debug!("Normalized to: '{}'", normalized);
                general_purpose::STANDARD.decode(&normalized)
            }),
            Box::new(|| {
                log::debug!("Trying normalized URL_SAFE");
                let normalized = Self::normalize_base64(encoded);
                log::debug!("Normalized to: '{}'", normalized);
                general_purpose::URL_SAFE.decode(&normalized)
            }),
        ];

        for (i, attempt) in decode_attempts.iter().enumerate() {
            match attempt() {
                Ok(result) => {
                    log::debug!(
                        "Successfully decoded with method {} - {} bytes",
                        i,
                        result.len()
                    );
                    return Ok(result);
                }
                Err(e) => {
                    log::debug!("Method {} failed: {}", i, e);
                }
            }
        }

        Err(ContactError::DecodeError(
            "All base64 decode methods failed".to_string(),
        ))
    }

    pub async fn get_contacts(&self) -> Vec<Contact> {
        let contacts = self.contacts.read().await;
        contacts.values().cloned().collect()
    }

    pub async fn get_contact_by_name(&self, name: &str) -> Option<Contact> {
        let contacts = self.contacts.read().await;
        contacts.values().find(|c| c.name == name).cloned()
    }

    pub async fn get_contact_by_id(&self, id: &str) -> Option<Contact> {
        let contacts = self.contacts.read().await;
        contacts.get(id).cloned()
    }

    pub async fn block_contact(&self, contact_id: &str) -> Result<(), ContactError> {
        log::info!("Blocking contact: {}", contact_id);

        let mut contacts = self.contacts.write().await;
        if let Some(contact) = contacts.get_mut(contact_id) {
            contact.status = ContactStatus::Blocked;
            log::info!("Contact '{}' blocked successfully", contact.name);
            Ok(())
        } else {
            Err(ContactError::NotFound(contact_id.to_string()))
        }
    }

    pub async fn unblock_contact(&self, contact_id: &str) -> Result<(), ContactError> {
        log::info!("Unblocking contact: {}", contact_id);

        let mut contacts = self.contacts.write().await;
        if let Some(contact) = contacts.get_mut(contact_id) {
            contact.status = ContactStatus::Offline;
            log::info!("Contact '{}' unblocked successfully", contact.name);
            Ok(())
        } else {
            Err(ContactError::NotFound(contact_id.to_string()))
        }
    }

    pub async fn update_contact_status(
        &self,
        contact_id: &str,
        status: ContactStatus,
    ) -> Result<(), ContactError> {
        let mut contacts = self.contacts.write().await;
        if let Some(contact) = contacts.get_mut(contact_id) {
            contact.status = status;
            contact.last_seen = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| ContactError::Crypto(e.to_string()))?
                .as_secs();
        }
        Ok(())
    }

    pub async fn remove_contact(&self, contact_id: &str) -> Result<(), ContactError> {
        log::info!("Removing contact: {}", contact_id);

        let mut contacts = self.contacts.write().await;
        if contacts.remove(contact_id).is_some() {
            log::info!("Contact removed successfully");
            Ok(())
        } else {
            Err(ContactError::NotFound(contact_id.to_string()))
        }
    }

    pub async fn set_trust_level(
        &self,
        contact_id: &str,
        trust_level: TrustLevel,
    ) -> Result<(), ContactError> {
        log::info!("Setting trust level for contact: {}", contact_id);

        let mut contacts = self.contacts.write().await;
        if let Some(contact) = contacts.get_mut(contact_id) {
            contact.trust_level = trust_level;
            log::info!("Trust level updated for '{}'", contact.name);
            Ok(())
        } else {
            Err(ContactError::NotFound(contact_id.to_string()))
        }
    }

    pub async fn load_contacts(
        &self,
        contacts: HashMap<String, Contact>,
    ) -> Result<(), ContactError> {
        log::info!("Loading {} contacts", contacts.len());

        let mut contacts_guard = self.contacts.write().await;
        *contacts_guard = contacts;

        Ok(())
    }

    pub async fn get_contacts_map(&self) -> HashMap<String, Contact> {
        let contacts = self.contacts.read().await;
        contacts.clone()
    }

    pub fn get_contacts_ref(&self) -> Arc<RwLock<HashMap<String, Contact>>> {
        self.contacts.clone()
    }
}
