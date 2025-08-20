mod frb_generated; /* AUTO INJECTED BY flutter_rust_bridge. This line may not be accurate, and you can change it according to your needs. */
pub mod api;
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

pub use api::*;
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
        AppConfig, AppEvent, ChatMessage, ConfigManager, Contact, ContactError, ContactManager,
        ContactStatus, CoreError, CryptoEvent, CryptoManager, EventBus, NetworkEvent,
        NetworkManager, Peer, PublicKey, ShadowGhostCore, StorageEvent, StorageManager,
        StorageStats,
    };
}
