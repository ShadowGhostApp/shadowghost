pub mod events;

use shadowghost::prelude::*;
use std::sync::Once;

static INIT: Once = Once::new();

pub fn init_test_logging() {
    INIT.call_once(|| {
        env_logger::Builder::from_env("RUST_LOG")
            .filter_level(log::LevelFilter::Debug)
            .is_test(true)
            .init();
    });
}

pub struct TestSetup {
    pub core: ShadowGhostCore,
    test_id: String,
}

impl TestSetup {
    pub async fn new(test_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let test_id = format!("{}-{}", test_name, uuid::Uuid::new_v4());

        let mut core = ShadowGhostCore::new_for_test(&test_id)?;


        core.initialize(Some(test_name.to_string())).await?;


        core.start_server().await?;


        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        Ok(Self { core, test_id })
    }

    pub fn get_event_receiver(&self) -> tokio::sync::broadcast::Receiver<AppEvent> {
        self.core.get_event_bus().subscribe()
    }

    pub async fn shutdown(self) -> Result<(), Box<dyn std::error::Error>> {
        self.core.shutdown().await?;


        let temp_dir = std::env::temp_dir()
            .join("shadowghost_test")
            .join(&self.test_id);
        if temp_dir.exists() {
            let _ = std::fs::remove_dir_all(&temp_dir);
        }

        Ok(())
    }


    pub async fn create_peer(name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Self::new(name).await
    }


    pub async fn get_contact_count(&self) -> usize {
        self.core.get_contact_count().await
    }


    pub async fn has_contact(&self, name: &str) -> bool {
        if let Ok(contacts) = self.core.get_contacts().await {
            contacts.iter().any(|c| c.name == name)
        } else {
            false
        }
    }


    pub async fn get_contact_names(&self) -> Vec<String> {
        if let Ok(contacts) = self.core.get_contacts().await {
            contacts.into_iter().map(|c| c.name).collect()
        } else {
            vec![]
        }
    }


    pub async fn send_test_message(
        &self,
        to: &str,
        content: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.core.send_message(to, content).await?;
        Ok(())
    }


    pub async fn get_message_count(&self, contact: &str) -> usize {
        if let Ok(messages) = self.core.get_chat_messages(contact).await {
            messages.len()
        } else {
            0
        }
    }


    pub async fn get_last_message(&self, contact: &str) -> Option<String> {
        if let Ok(messages) = self.core.get_chat_messages(contact).await {
            messages.last().map(|m| m.content.clone())
        } else {
            None
        }
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
        contact: &str,
        expected_count: usize,
        timeout_ms: u64,
    ) -> bool {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);

        while start.elapsed() < timeout {
            if self.get_message_count(contact).await == expected_count {
                return true;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        false
    }


    pub fn get_test_dir(&self) -> std::path::PathBuf {
        std::env::temp_dir()
            .join("shadowghost_test")
            .join(&self.test_id)
    }


    pub async fn is_server_running(&self) -> bool {
        self.core.is_server_started()
    }


    pub fn is_initialized(&self) -> bool {
        self.core.is_initialized()
    }


    pub async fn get_user_name(&self) -> Option<String> {
        if let Some(peer_info) = self.core.get_peer_info().await {

            if let Some(paren_pos) = peer_info.find(" (") {
                Some(peer_info[..paren_pos].to_string())
            } else {
                Some(peer_info)
            }
        } else {
            None
        }
    }


    pub async fn debug_info(&self) -> String {
        let contacts = self.get_contact_count().await;
        let user_name = self.get_user_name().await.unwrap_or("Unknown".to_string());
        let initialized = self.is_initialized();
        let server_running = self.is_server_running().await;

        format!(
            "TestSetup[{}]: user={}, contacts={}, initialized={}, server={}",
            self.test_id, user_name, contacts, initialized, server_running
        )
    }
}

impl Drop for TestSetup {
    fn drop(&mut self) {

        let temp_dir = std::env::temp_dir()
            .join("shadowghost_test")
            .join(&self.test_id);
        if temp_dir.exists() {
            let _ = std::fs::remove_dir_all(&temp_dir);
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


pub async fn connect_all(setups: &[TestSetup]) -> Result<(), Box<dyn std::error::Error>> {
    let links: Vec<String> = {
        let mut links = Vec::new();
        for setup in setups {
            links.push(setup.core.generate_sg_link().await?);
        }
        links
    };


    for (i, setup) in setups.iter().enumerate() {
        for (j, link) in links.iter().enumerate() {
            if i != j {

                if let Err(e) = setup.core.add_contact_by_sg_link(link).await {

                    if !e.to_string().contains("Cannot add yourself") {
                        return Err(e.into());
                    }
                }
            }
        }
    }


    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

    Ok(())
}


pub async fn shutdown_all(setups: Vec<TestSetup>) -> Result<(), Box<dyn std::error::Error>> {
    for setup in setups {
        setup.shutdown().await?;
    }
    Ok(())
}


pub fn assert_contact_exists(contacts: &[shadowghost::Contact], name: &str) {
    assert!(
        contacts.iter().any(|c| c.name == name),
        "Contact '{}' not found in contacts: {:?}",
        name,
        contacts.iter().map(|c| &c.name).collect::<Vec<_>>()
    );
}

pub fn assert_contact_count(contacts: &[shadowghost::Contact], expected: usize) {
    assert_eq!(
        contacts.len(),
        expected,
        "Expected {} contacts, found {}: {:?}",
        expected,
        contacts.len(),
        contacts.iter().map(|c| &c.name).collect::<Vec<_>>()
    );
}

pub fn assert_message_exists(messages: &[shadowghost::ChatMessage], content: &str) {
    assert!(
        messages.iter().any(|m| m.content == content),
        "Message '{}' not found in messages: {:?}",
        content,
        messages.iter().map(|m| &m.content).collect::<Vec<_>>()
    );
}

pub fn assert_message_count(messages: &[shadowghost::ChatMessage], expected: usize) {
    assert_eq!(
        messages.len(),
        expected,
        "Expected {} messages, found {}: {:?}",
        expected,
        messages.len(),
        messages.iter().map(|m| &m.content).collect::<Vec<_>>()
    );
}
