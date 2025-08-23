pub mod api;
pub mod contacts;
pub mod core;
pub mod crypto;
pub mod data;
pub mod events;
mod frb_generated; /* AUTO INJECTED BY flutter_rust_bridge. This line may not be accurate, and you can change it according to your needs. */
pub mod network;
pub mod ui;
pub mod utils;

pub use api::*;
pub use contacts::{ContactError, ContactManager}
pub use core::{AppConfig, ConfigManager, CoreError, Peer, ShadowGhostCore};
pub use data::{StorageManager, StorageStats};
pub use events::{AppEvent, CryptoEvent, EventBus, NetworkEvent, StorageEvent};
pub use network::{
    ChatMessage, ChatMessageType, Contact, ContactStatus, DeliveryStatus, NetworkManager,
    NetworkStats, PeerData, TrustLevel,
};
pub use crypto::{CryptoManager, EncryptedMessage, PublicKey};

pub mod prelude {
    pub use crate::{
        AppConfig, AppEvent, ChatMessage, ChatMessageType, ConfigManager, Contact, ContactError,
        ContactManager, ContactStatus, CoreError, CryptoEvent, CryptoManager, DeliveryStatus,
        EventBus, NetworkEvent, NetworkManager, Peer, PublicKey, ShadowGhostCore, StorageEvent,
        StorageManager, StorageStats, TrustLevel,
    };
}
