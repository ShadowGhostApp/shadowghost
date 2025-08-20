use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub id: String,
    pub name: String,
    pub address: String,
    pub public_key: Vec<u8>,
    pub port: u16,
}

impl Peer {
    pub fn new(name: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            address: "127.0.0.1".to_string(),
            public_key: vec![],
            port: 8080,
        }
    }

    pub fn with_address(name: String, address: String, port: u16) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            address,
            public_key: vec![],
            port,
        }
    }

    pub fn get_full_address(&self) -> String {
        format!("{}:{}", self.address, self.port)
    }

    pub fn update_address(&mut self, address: String, port: u16) {
        self.address = address;
        self.port = port;
    }

    pub fn set_public_key(&mut self, key: Vec<u8>) {
        self.public_key = key;
    }
}

impl fmt::Display for Peer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({}:{})", self.name, self.address, self.port)
    }
}
