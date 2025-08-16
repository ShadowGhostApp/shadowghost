use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkEvent {
    ServerStarted { port: u16 },
    ServerStopped,
    MessageReceived { from: String, content: String },
    ContactAdded { name: String },
    Error { error: String },
    Debug { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageEvent {
    ContactsSaved { count: usize },
    Error { error: String, operation: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CryptoEvent {
    KeyPairGenerated,
    Error { error: String, operation: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppEvent {
    Network(NetworkEvent),
    Storage(StorageEvent),
    Crypto(CryptoEvent),
}

pub type EventSender = broadcast::Sender<AppEvent>;
pub type EventReceiver = broadcast::Receiver<AppEvent>;

#[derive(Clone, Debug)]
pub struct EventBus {
    sender: EventSender,
}

impl EventBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1000);
        Self { sender }
    }

    pub fn subscribe(&self) -> EventReceiver {
        self.sender.subscribe()
    }

    pub fn emit(&self, event: AppEvent) {
        let _ = self.sender.send(event);
    }

    pub fn emit_network(&self, event: NetworkEvent) {
        self.emit(AppEvent::Network(event));
    }

    pub fn emit_storage(&self, event: StorageEvent) {
        self.emit(AppEvent::Storage(event));
    }

    pub fn emit_crypto(&self, event: CryptoEvent) {
        self.emit(AppEvent::Crypto(event));
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
