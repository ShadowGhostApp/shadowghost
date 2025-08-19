use crate::crypto::CryptoManager;
use crate::events::EventBus;
use crate::network::{Contact, ContactStatus, PeerData, TrustLevel};
use crate::peer::Peer;
use crate::protocol::MessageType;
use base64::{engine::general_purpose, Engine as _};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
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

#[derive(serde::Serialize, serde::Deserialize)]
struct PingMessage {
    message_type: MessageType,
    sender_id: String,
    recipient_id: String,
    content: Vec<u8>,
    timestamp: u64,
    message_id: Option<String>,
}

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

        let encoded = general_purpose::URL_SAFE_NO_PAD.encode(data.as_bytes());

        match general_purpose::URL_SAFE_NO_PAD.decode(&encoded) {
            Ok(test_decoded) => match String::from_utf8(test_decoded) {
                Ok(test_json) => {
                    if test_json != data {
                        return Err(ContactError::ParseError(
                            "Link generation self-test failed".to_string(),
                        ));
                    }
                }
                Err(_e) => {
                    return Err(ContactError::ParseError(
                        "Link generation produces invalid UTF-8".to_string(),
                    ));
                }
            },
            Err(_e) => {
                return Err(ContactError::ParseError(
                    "Link generation produces invalid base64".to_string(),
                ));
            }
        }

        let sg_link = format!("sg://{}", encoded);
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
        if !sg_link.starts_with("sg://") {
            return Err(ContactError::InvalidLinkFormat(
                "Link must start with sg://".to_string(),
            ));
        }

        let encoded = &sg_link[5..];

        let valid_chars = encoded.chars().all(|c| {
            c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=' || c == '-' || c == '_'
        });

        if !valid_chars {
            return Err(ContactError::InvalidLinkFormat(
                "Contains invalid base64 characters".to_string(),
            ));
        }

        let decoded_bytes = self.try_decode_base64(encoded)?;

        if decoded_bytes.is_empty() || decoded_bytes[0] != b'{' {
            return Err(ContactError::DecodeError(
                "Data is not JSON format".to_string(),
            ));
        }

        match std::str::from_utf8(&decoded_bytes) {
            Ok(data_str) => {
                let peer_data: PeerData = serde_json::from_str(data_str)
                    .map_err(|e| ContactError::ParseError(format!("JSON parse failed: {}", e)))?;

                if peer_data.id == self.local_peer.id {
                    return Err(ContactError::DuplicateContact(
                        "Cannot add yourself as contact".to_string(),
                    ));
                }

                let contact = Contact {
                    id: peer_data.id.clone(),
                    name: peer_data.name.clone(),
                    address: peer_data.address.clone(),
                    status: ContactStatus::Offline,
                    trust_level: TrustLevel::Low,
                    last_seen: peer_data.connected_at,
                };

                {
                    let mut contacts = self.contacts.write().await;
                    if contacts.contains_key(&peer_data.id) {
                        // Contact already exists, update it
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

                self.event_bus
                    .emit_network(crate::events::NetworkEvent::ContactAdded {
                        contact: contact.clone(),
                    });

                Ok(contact)
            }
            Err(utf8_error) => Err(ContactError::DecodeError(format!(
                "UTF-8 conversion failed: {}. Decoded {} bytes, invalid sequence at position {}",
                utf8_error,
                decoded_bytes.len(),
                utf8_error.valid_up_to()
            ))),
        }
    }

    fn try_decode_base64(&self, encoded: &str) -> Result<Vec<u8>, ContactError> {
        let decode_attempts: Vec<Box<dyn Fn() -> Result<Vec<u8>, base64::DecodeError>>> = vec![
            Box::new(|| general_purpose::URL_SAFE_NO_PAD.decode(encoded)),
            Box::new(|| general_purpose::URL_SAFE.decode(encoded)),
            Box::new(|| general_purpose::STANDARD_NO_PAD.decode(encoded)),
            Box::new(|| general_purpose::STANDARD.decode(encoded)),
            Box::new(|| {
                let normalized = Self::normalize_base64(encoded);
                general_purpose::STANDARD.decode(&normalized)
            }),
            Box::new(|| {
                let normalized = Self::normalize_base64(encoded);
                general_purpose::URL_SAFE.decode(&normalized)
            }),
        ];

        for attempt in decode_attempts.iter() {
            if let Ok(result) = attempt() {
                return Ok(result);
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

    pub async fn check_contact_online(&self, contact_name: &str) -> bool {
        if let Some(contact) = self.get_contact_by_name(contact_name).await {
            self.ping_contact(&contact).await
        } else {
            false
        }
    }

    async fn ping_contact(&self, contact: &Contact) -> bool {
        let ping_message = PingMessage {
            message_type: MessageType::Ping,
            sender_id: self.local_peer.id.clone(),
            recipient_id: contact.id.clone(),
            content: vec![],
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            message_id: None,
        };

        match self
            .send_ping_and_wait_for_pong(&contact.address, &ping_message)
            .await
        {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    async fn send_ping_and_wait_for_pong(
        &self,
        address: &str,
        ping_message: &PingMessage,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let stream_result =
            tokio::time::timeout(Duration::from_secs(3), TcpStream::connect(address)).await;

        let mut stream = match stream_result {
            Ok(Ok(s)) => s,
            Ok(Err(_)) => return Err("Connection failed".into()),
            Err(_) => return Err("Connection timeout".into()),
        };

        let data = serde_json::to_vec(ping_message)?;

        match tokio::time::timeout(Duration::from_secs(2), stream.write_all(&data)).await {
            Ok(Ok(_)) => {
                let _ = stream.flush().await;

                let mut buffer = [0; 1024];
                match tokio::time::timeout(Duration::from_secs(3), stream.read(&mut buffer)).await {
                    Ok(Ok(n)) if n > 0 => {
                        if let Ok(response) = serde_json::from_slice::<PingMessage>(&buffer[..n]) {
                            if response.message_type == MessageType::Pong {
                                return Ok(());
                            }
                        }
                        Err("Invalid pong response".into())
                    }
                    _ => Err("No pong received".into()),
                }
            }
            _ => Err("Failed to send ping".into()),
        }
    }

    pub async fn block_contact(&self, contact_id: &str) -> Result<(), ContactError> {
        let mut contacts = self.contacts.write().await;
        if let Some(contact) = contacts.get_mut(contact_id) {
            contact.status = ContactStatus::Blocked;
            Ok(())
        } else {
            Err(ContactError::NotFound(contact_id.to_string()))
        }
    }

    pub async fn unblock_contact(&self, contact_id: &str) -> Result<(), ContactError> {
        let mut contacts = self.contacts.write().await;
        if let Some(contact) = contacts.get_mut(contact_id) {
            contact.status = ContactStatus::Offline;
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
        let mut contacts = self.contacts.write().await;
        if contacts.remove(contact_id).is_some() {
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
        let mut contacts = self.contacts.write().await;
        if let Some(contact) = contacts.get_mut(contact_id) {
            contact.trust_level = trust_level;
            Ok(())
        } else {
            Err(ContactError::NotFound(contact_id.to_string()))
        }
    }

    pub async fn load_contacts(
        &self,
        contacts: HashMap<String, Contact>,
    ) -> Result<(), ContactError> {
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
