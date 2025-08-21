pub mod api;
pub mod core;
pub mod data;
pub mod events;
pub mod generated;
pub mod network;
pub mod security;
pub mod ui;
pub mod utils;

pub use api::*;
pub use core::{AppConfig, ConfigManager, CoreError, Peer, ShadowGhostCore};
pub use data::{ContactError, ContactManager, StorageManager, StorageStats};
pub use events::{AppEvent, CryptoEvent, EventBus, NetworkEvent, StorageEvent};
pub use network::{
    ChatMessage, ChatMessageType, Contact, ContactStatus, DeliveryStatus, NetworkManager,
    NetworkStats, PeerData, TrustLevel,
};
pub use security::{CryptoManager, EncryptedMessage, PublicKey};

pub mod prelude {
    pub use crate::{
        AppConfig, AppEvent, ChatMessage, ChatMessageType, ConfigManager, Contact, ContactError,
        ContactManager, ContactStatus, CoreError, CryptoEvent, CryptoManager, DeliveryStatus,
        EventBus, NetworkEvent, NetworkManager, Peer, PublicKey, ShadowGhostCore, StorageEvent,
        StorageManager, StorageStats, TrustLevel,
    };
}
