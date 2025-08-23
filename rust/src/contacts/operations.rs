use crate::core::peer::Peer;
use crate::network::{Contact, ContactStatus, PeerData, TrustLevel};
use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;

use super::error::ContactError;

pub fn generate_sg_link(peer: &Peer) -> Result<String, ContactError> {
    let peer_data = PeerData {
        id: peer.id.clone(),
        name: peer.name.clone(),
        address: peer.get_full_address(),
        public_key: peer.public_key.clone(),
        connected_at: Utc::now(),
        last_seen: Utc::now(),
        bytes_sent: 0,
        bytes_received: 0,
    };

    let json_data = serde_json::to_string(&peer_data)
        .map_err(|e| ContactError::SerializationError(e.to_string()))?;

    let encoded = general_purpose::STANDARD.encode(json_data);
    Ok(format!("sg://{}", encoded))
}

pub fn parse_sg_link(sg_link: &str, current_peer_name: &str) -> Result<Contact, ContactError> {
    if !sg_link.starts_with("sg://") {
        return Err(ContactError::InvalidContact(
            "Invalid SG link format".to_string(),
        ));
    }

    let link_data = &sg_link[5..];

    let decoded_data = general_purpose::STANDARD
        .decode(link_data)
        .map_err(|e| ContactError::InvalidContact(format!("Decode error: {}", e)))?;

    let data_str = String::from_utf8(decoded_data)
        .map_err(|_| ContactError::InvalidContact("UTF-8 conversion failed".to_string()))?;

    let peer_data: PeerData = serde_json::from_str(&data_str)
        .map_err(|_| ContactError::InvalidContact("JSON parse failed".to_string()))?;

    if peer_data.name == current_peer_name {
        return Err(ContactError::InvalidContact(
            "Cannot add yourself as contact".to_string(),
        ));
    }

    let contact = Contact {
        id: peer_data.id,
        name: peer_data.name,
        address: peer_data.address,
        status: ContactStatus::Offline,
        trust_level: TrustLevel::Pending,
        last_seen: Some(peer_data.last_seen),
    };

    Ok(contact)
}

pub fn create_contact_from_peer_data(peer_data: &PeerData) -> Contact {
    Contact {
        id: peer_data.id.clone(),
        name: peer_data.name.clone(),
        address: peer_data.address.clone(),
        status: ContactStatus::Offline,
        trust_level: TrustLevel::Pending,
        last_seen: Some(peer_data.last_seen),
    }
}
