pub mod bridge;
pub mod cli;
pub mod config;
pub mod contact_manager;
pub mod core;
pub mod crypto;
pub mod events;
pub mod network;
pub mod network_discovery;
pub mod peer;
pub mod protocol;
pub mod storage;
pub mod tls_masking;
pub mod utils;

pub use bridge::*;
pub use config::{AppConfig, ConfigManager};
pub use contact_manager::{ContactError, ContactManager};
pub use core::{CoreError, ShadowGhostCore};
pub use crypto::{CryptoManager, EncryptedMessage, PublicKey};
pub use events::{AppEvent, CryptoEvent, EventBus, NetworkEvent, StorageEvent};
pub use network::{
    ChatMessage, ChatMessageType, Contact, ContactStatus, DeliveryStatus, NetworkManager,
    NetworkStats, PeerData, TrustLevel,
};
pub use peer::Peer;
pub use storage::{StorageManager, StorageStats};

pub mod prelude {
    pub use crate::{
        AppConfig, AppEvent, ChatMessage, ChatMessageType, ConfigManager, Contact, ContactError,
        ContactManager, ContactStatus, CoreError, CryptoEvent, CryptoManager, DeliveryStatus,
        EventBus, NetworkEvent, NetworkManager, Peer, PublicKey, ShadowGhostCore, StorageEvent,
        StorageManager, StorageStats, TrustLevel,
    };

    pub use chrono;
    pub use uuid;
}

flutter_rust_bridge::frb_generated_boilerplate!();
