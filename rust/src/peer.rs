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

    pub fn get_info(&self) -> String {
        format!("{} ({})", self.name, self.address)
    }
}
