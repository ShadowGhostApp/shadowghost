pub mod chats;
pub mod contacts;
pub mod core;
pub mod crypto;
pub mod network;
pub mod storage;
pub mod utils;

// Re-export Flutter API functions
pub use chats::flutter_api::*;
pub use contacts::flutter_api::*;
pub use core::flutter_api::*;
pub use crypto::flutter_api::*;
pub use network::flutter_api::*;
pub use storage::flutter_api::*;

// Re-export core types
pub use chats::types::*;
pub use contacts::types::*;
pub use core::engine::ENGINE;
pub use crypto::types::*;
pub use network::types::*;
pub use storage::types::*;
