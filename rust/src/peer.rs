use rand::Rng;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct Peer {
    pub id: String,
    pub name: String,
    pub address: String,
}

impl Peer {
    pub fn new(name: String, address: String) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        Self { id, name, address }
    }

    pub fn new_with_entropy(name: String, address: String) -> Self {
        let mut hasher = DefaultHasher::new();

        std::process::id().hash(&mut hasher);
        std::thread::current().id().hash(&mut hasher);
        name.hash(&mut hasher);
        address.hash(&mut hasher);

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        timestamp.hash(&mut hasher);

        let random_bytes: [u8; 16] = rand::rng().random();
        random_bytes.hash(&mut hasher);

        let hash_value = hasher.finish();

        let uuid_bytes = [
            (hash_value >> 56) as u8,
            (hash_value >> 48) as u8,
            (hash_value >> 40) as u8,
            (hash_value >> 32) as u8,
            (hash_value >> 24) as u8,
            (hash_value >> 16) as u8,
            (hash_value >> 8) as u8,
            hash_value as u8,
            random_bytes[8],
            random_bytes[9],
            random_bytes[10],
            random_bytes[11],
            random_bytes[12],
            random_bytes[13],
            random_bytes[14],
            random_bytes[15],
        ];

        let base_uuid = uuid::Uuid::from_bytes(uuid_bytes);
        let final_uuid = uuid::Uuid::new_v4();

        let combined = format!(
            "{}-{}",
            &base_uuid.to_string()[..18],
            &final_uuid.to_string()[19..]
        );

        Self {
            id: combined,
            name,
            address,
        }
    }

    pub fn get_info(&self) -> String {
        format!("{} ({})", self.name, self.address)
    }

    pub fn get_short_id(&self) -> String {
        if self.id.len() >= 8 {
            self.id[..8].to_string()
        } else {
            self.id.clone()
        }
    }
}
