pub mod chats;
pub mod chats;
pub mod contacts;
pub mod core;
pub mod crypto;
pub mod events;
pub mod network;
pub mod storage;
pub mod ui;
pub mod utils;

// Re-export Flutter API functions
pub use chats::flutter_api::{create_chat, get_all_chats, get_chat, get_messages, send_message};
pub use contacts::flutter_api::*;
pub use crypto::flutter_api::*;
pub use network::flutter_api::*;
pub use network::flutter_api::{get_network_stats, start_network_server, stop_network_server};
pub use storage::flutter_api::*;

// Re-export core types
pub use core::engine::ENGINE;
