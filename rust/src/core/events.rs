use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub enum Event {
    MessageReceived { from: String, content: String },
    ContactAdded { contact_id: String },
    NetworkStateChanged { online: bool },
}

#[derive(Clone)]
pub struct EventBus {
    sender: broadcast::Sender<Event>,
}

impl EventBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1000);
        Self { sender }
    }

    pub fn emit(&self, event: Event) {
        let _ = self.sender.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.sender.subscribe()
    }
}
