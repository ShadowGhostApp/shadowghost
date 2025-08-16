use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]

pub enum MessageType {
    TextMessage,

    FileShare,

    VoiceCall,

    Handshake,
}

#[derive(Debug, Serialize, Deserialize)]

pub struct Message {
    pub from: String,

    pub to: String,

    pub msg_type: MessageType,

    pub content: Vec<u8>,

    pub timestamp: u64,
}
