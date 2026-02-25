pub mod events;

use shadowghost::contacts::ContactManager;
use shadowghost::core::{Peer};
use shadowghost::events::{AppEvent, EventBus};
use shadowghost::network::{ChatMessage, Contact, ContactStatus, TrustLevel};
use shadowghost::storage::StorageManager;
use std::sync::Once;

static INIT: Once = Once::new();

pub fn init_test_logging() {
    INIT.call_once(|| {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
            .is_test(true)
            .try_init()
            .ok();
    });
}

pub struct TestSetup {
    pub temp_dir: std::path::PathBuf,
    pub contact_manager: ContactManager,
    pub storage_manager: StorageManager,
    pub event_bus: EventBus,
    pub peer: Peer,
    test_id: String,
}

impl TestSetup {
    pub async fn new(test_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let test_id = format!("{}-{}", test_name, uuid::Uuid::new_v4());
        let temp_dir = std::env::temp_dir().join("shadowghost_test").join(&test_id);
        std::fs::create_dir_all(&temp_dir)?;

        let event_bus = EventBus::new();
        let storage_manager = StorageManager::new(&temp_dir, event_bus.clone())?;
        storage_manager.initialize().await?;

        let contact_manager = ContactManager::new(&temp_dir)?;

        let port = 8000 + (rand::random::<u16>() % 1000);
        let peer = Peer::new(test_name.to_string(), format!("127.0.0.1:{}", port));

        // Wait a bit to ensure everything is initialized
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        Ok(Self {
            temp_dir,
            contact_manager,
            storage_manager,
            event_bus,
            peer,
            test_id,
        })
    }

    pub fn get_event_receiver(&self) -> tokio::sync::broadcast::Receiver<AppEvent> {
        self.event_bus.subscribe()
    }

    pub async fn shutdown(self) -> Result<(), Box<dyn std::error::Error>> {
        if self.temp_dir.exists() {
            std::fs::remove_dir_all(&self.temp_dir)?;
        }
        Ok(())
    }

    pub async fn create_peer(name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Self::new(name).await
    }

    pub async fn get_contact_count(&self) -> usize {
        self.contact_manager.get_contacts().len()
    }

    pub async fn has_contact(&self, name: &str) -> bool {
        self.contact_manager
            .get_contacts()
            .iter()
            .any(|c| c.name == name)
    }

    pub async fn get_contact_names(&self) -> Vec<String> {
        self.contact_manager
            .get_contacts()
            .into_iter()
            .map(|c| c.name)
            .collect()
    }

    pub async fn add_test_contact(
        &mut self,
        name: &str,
    ) -> Result<Contact, Box<dyn std::error::Error>> {
        let contact = Contact {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            address: format!("127.0.0.1:{}", 8000 + rand::random::<u16>() % 1000),
            status: ContactStatus::Offline,
            trust_level: TrustLevel::Unknown,
            last_seen: Some(chrono::Utc::now()),
        };

        self.contact_manager.add_contact(contact.clone())?;
        Ok(contact)
    }

    pub async fn send_test_message(
        &self,
        from: &str,
        to: &str,
        content: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let message = ChatMessage {
            id: uuid::Uuid::new_v4().to_string(),
            from: from.to_string(),
            to: to.to_string(),
            content: content.to_string(),
            msg_type: shadowghost::network::ChatMessageType::Text,
            timestamp: chrono::Utc::now().timestamp() as u64,
            delivery_status: shadowghost::network::DeliveryStatus::Sent,
        };

        let chat_id = format!("{}_{}", from, to);
        self.storage_manager
            .save_message(&chat_id, &message)
            .await?;
        Ok(message.id)
    }

    pub async fn get_message_count(&self, chat_id: &str) -> usize {
        self.storage_manager
            .get_messages(chat_id)
            .await
            .map(|messages| messages.len())
            .unwrap_or(0)
    }

    pub async fn get_last_message(&self, chat_id: &str) -> Option<String> {
        self.storage_manager
            .get_messages(chat_id)
            .await
            .ok()
            .and_then(|messages| messages.last().map(|m| m.content.clone()))
    }

    pub async fn wait_for_contacts(&self, expected_count: usize, timeout_ms: u64) -> bool {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);

        while start.elapsed() < timeout {
            if self.get_contact_count().await == expected_count {
                return true;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        false
    }

    pub async fn wait_for_messages(
        &self,
        chat_id: &str,
        expected_count: usize,
        timeout_ms: u64,
    ) -> bool {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);

        while start.elapsed() < timeout {
            if self.get_message_count(chat_id).await == expected_count {
                return true;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        false
    }

    pub fn get_test_dir(&self) -> std::path::PathBuf {
        self.temp_dir.clone()
    }

    pub fn get_user_name(&self) -> String {
        self.peer.name.clone()
    }

    pub async fn debug_info(&self) -> String {
        let contacts = self.get_contact_count().await;
        let user_name = self.get_user_name();

        format!(
            "TestSetup[{}]: user={}, contacts={}",
            self.test_id, user_name, contacts
        )
    }

    pub async fn generate_sg_link(&self) -> Result<String, Box<dyn std::error::Error>> {
        use shadowghost::contacts::generate_sg_link;
        Ok(generate_sg_link(&self.peer)?)
    }

    pub async fn add_contact_by_sg_link(
        &mut self,
        sg_link: &str,
    ) -> Result<Contact, Box<dyn std::error::Error>> {
        use shadowghost::contacts::parse_sg_link;
        let contact = parse_sg_link(sg_link, &self.peer.name)?;
        self.contact_manager.add_contact(contact.clone())?;
        Ok(contact)
    }
}

impl Drop for TestSetup {
    fn drop(&mut self) {
        if self.temp_dir.exists() {
            let _ = std::fs::remove_dir_all(&self.temp_dir);
        }
    }
}

pub async fn create_test_group(
    names: &[&str],
) -> Result<Vec<TestSetup>, Box<dyn std::error::Error>> {
    let mut setups = Vec::new();

    for &name in names {
        let setup = TestSetup::new(name).await?;
        setups.push(setup);
    }

    Ok(setups)
}

pub async fn connect_all(setups: &mut [TestSetup]) -> Result<(), Box<dyn std::error::Error>> {
    let links: Vec<String> = {
        let mut links = Vec::new();
        for setup in setups.iter() {
            links.push(setup.generate_sg_link().await?);
        }
        links
    };

    for (i, setup) in setups.iter_mut().enumerate() {
        for (j, link) in links.iter().enumerate() {
            if i != j {
                if let Err(e) = setup.add_contact_by_sg_link(link).await {
                    if !e.to_string().contains("Cannot add yourself") {
                        return Err(e);
                    }
                }
            }
        }
    }

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    Ok(())
}

pub async fn shutdown_all(setups: Vec<TestSetup>) -> Result<(), Box<dyn std::error::Error>> {
    for setup in setups {
        setup.shutdown().await?;
    }
    Ok(())
}

pub fn assert_contact_exists(contacts: &[Contact], name: &str) {
    assert!(
        contacts.iter().any(|c| c.name == name),
        "Contact '{}' not found in contacts: {:?}",
        name,
        contacts.iter().map(|c| &c.name).collect::<Vec<_>>()
    );
}

pub fn assert_contact_count(contacts: &[Contact], expected: usize) {
    assert_eq!(
        contacts.len(),
        expected,
        "Expected {} contacts, found {}: {:?}",
        expected,
        contacts.len(),
        contacts.iter().map(|c| &c.name).collect::<Vec<_>>()
    );
}

pub fn assert_message_exists(messages: &[ChatMessage], content: &str) {
    assert!(
        messages.iter().any(|m| m.content == content),
        "Message '{}' not found in messages: {:?}",
        content,
        messages.iter().map(|m| &m.content).collect::<Vec<_>>()
    );
}

pub fn assert_message_count(messages: &[ChatMessage], expected: usize) {
    assert_eq!(
        messages.len(),
        expected,
        "Expected {} messages, found {}: {:?}",
        expected,
        messages.len(),
        messages.iter().map(|m| &m.content).collect::<Vec<_>>()
    );
}

#[macro_export]
macro_rules! assert_eventually {
    ($condition:expr, $timeout_ms:expr) => {{
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis($timeout_ms);

        loop {
            if $condition {
                break;
            }

            if start.elapsed() >= timeout {
                panic!("Condition was not met within {}ms", $timeout_ms);
            }

            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    }};
}

pub use assert_eventually;
