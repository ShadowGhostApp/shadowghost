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
    pub fn new(name: String, address: String) -> Self {
        let (host, port) = Self::parse_address(&address);
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            address: host,
            public_key: vec![],
            port,
        }
    }

    pub fn new_with_entropy(name: String, address: String) -> Self {
        let (host, port) = Self::parse_address(&address);


        let entropy = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let id = format!("{}_{}", uuid::Uuid::new_v4(), entropy);

        Self {
            id,
            name,
            address: host,
            public_key: vec![],
            port,
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

    pub fn get_short_id(&self) -> String {
        if self.id.len() > 8 {
            self.id[..8].to_string()
        } else {
            self.id.clone()
        }
    }

    pub fn get_info(&self) -> String {
        format!("{} ({}:{})", self.name, self.address, self.port)
    }

    fn parse_address(address: &str) -> (String, u16) {
        if let Some(colon_pos) = address.rfind(':') {
            let host = &address[..colon_pos];
            let port_str = &address[colon_pos + 1..];

            if let Ok(port) = port_str.parse::<u16>() {
                (host.to_string(), port)
            } else {
                (address.to_string(), 8080)
            }
        } else {
            (address.to_string(), 8080)
        }
    }
}

impl fmt::Display for Peer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({}:{})", self.name, self.address, self.port)
    }
}
