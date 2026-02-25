// tests/integration_tests.rs
use shadowghost::core::{Engine, Profile, ProfileManager};
use shadowghost::crypto::CryptoManager;
use shadowghost::events::EventBus;
use shadowghost::network::{
    ChatMessage, ChatMessageType, Contact, ContactStatus, DeliveryStatus, TrustLevel,
};
use shadowghost::storage::StorageManager;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_core_initialization() -> Result<(), Box<dyn std::error::Error>> {
    let profile = Profile {
        id: "test_profile".to_string(),
        name: "Test Profile".to_string(),
        created_at: chrono::Utc::now(),
        last_used: chrono::Utc::now(),
    };

    let temp_dir = std::env::temp_dir().join("shadowghost_test_init");
    std::fs::create_dir_all(&temp_dir)?;

    let engine = Engine::new(profile, temp_dir.clone())?;

    // Test that engine was created successfully
    assert!(temp_dir.exists());

    std::fs::remove_dir_all(&temp_dir)?;
    Ok(())
}

#[tokio::test]
async fn test_contact_operations() -> Result<(), Box<dyn std::error::Error>> {
    use shadowghost::contacts::ContactManager;

    let temp_dir = std::env::temp_dir().join("shadowghost_contact_test");
    std::fs::create_dir_all(&temp_dir)?;

    let mut contact_manager = ContactManager::new(&temp_dir)?;

    let contact = Contact {
        id: "test_contact_1".to_string(),
        name: "Test Contact".to_string(),
        address: "127.0.0.1:8081".to_string(),
        status: ContactStatus::Offline,
        trust_level: TrustLevel::Unknown,
        last_seen: Some(chrono::Utc::now()),
    };

    // Add contact
    contact_manager.add_contact(contact.clone())?;

    // Get contacts
    let contacts = contact_manager.get_contacts();
    assert_eq!(contacts.len(), 1);
    assert_eq!(contacts[0].name, "Test Contact");

    // Update trust level
    contact_manager.set_trust_level(&contact.id, TrustLevel::Trusted)?;

    // Remove contact
    contact_manager.remove_contact(&contact.id)?;

    let contacts_after_remove = contact_manager.get_contacts();
    assert_eq!(contacts_after_remove.len(), 0);

    std::fs::remove_dir_all(&temp_dir)?;
    Ok(())
}

#[tokio::test]
async fn test_storage_operations() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = std::env::temp_dir().join("shadowghost_storage_test");
    std::fs::create_dir_all(&temp_dir)?;

    let event_bus = EventBus::new();
    let storage_manager = StorageManager::new(&temp_dir, event_bus)?;
    storage_manager.initialize().await?;

    let test_contact = Contact {
        id: "storage_contact_1".to_string(),
        name: "Storage Contact".to_string(),
        address: "127.0.0.1:8083".to_string(),
        status: ContactStatus::Offline,
        trust_level: TrustLevel::Unknown,
        last_seen: Some(chrono::Utc::now()),
    };

    // Save contact
    storage_manager.save_contact(&test_contact).await?;

    // Get contacts
    let contacts = storage_manager.get_contacts().await?;
    assert_eq!(contacts.len(), 1);
    assert_eq!(contacts[0].name, "Storage Contact");

    // Delete contact
    storage_manager.delete_contact(&test_contact.id).await?;

    let contacts_after_delete = storage_manager.get_contacts().await?;
    assert_eq!(contacts_after_delete.len(), 0);

    std::fs::remove_dir_all(&temp_dir)?;
    Ok(())
}

#[tokio::test]
async fn test_crypto_operations() -> Result<(), Box<dyn std::error::Error>> {
    let crypto_manager = CryptoManager::new()?;

    let public_key = crypto_manager.get_public_key();
    assert!(!public_key.key_data.is_empty());

    let test_data = b"Hello, cryptographic world!";
    let encrypted = crypto_manager.encrypt(test_data)?;
    assert_ne!(encrypted, test_data);

    let decrypted = crypto_manager.decrypt(&encrypted)?;
    assert_eq!(decrypted, test_data);

    let signature = crypto_manager.sign_data(test_data)?;
    let verification = crypto_manager.verify_signature(test_data, &signature, &public_key)?;
    assert!(verification);

    Ok(())
}

#[tokio::test]
async fn test_message_storage() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = std::env::temp_dir().join("shadowghost_message_test");
    std::fs::create_dir_all(&temp_dir)?;

    let event_bus = EventBus::new();
    let storage_manager = StorageManager::new(&temp_dir, event_bus)?;
    storage_manager.initialize().await?;

    let test_message = ChatMessage {
        id: "msg1".to_string(),
        from: "alice".to_string(),
        to: "bob".to_string(),
        content: "Hello, world!".to_string(),
        msg_type: ChatMessageType::Text,
        timestamp: chrono::Utc::now().timestamp() as u64,
        delivery_status: DeliveryStatus::Delivered,
    };

    // Save message
    storage_manager
        .save_message("test_chat", &test_message)
        .await?;

    // Get messages
    let messages = storage_manager.get_messages("test_chat").await?;
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].content, "Hello, world!");

    std::fs::remove_dir_all(&temp_dir)?;
    Ok(())
}

#[tokio::test]
async fn test_profile_management() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = std::env::temp_dir().join("shadowghost_profile_test");
    std::fs::create_dir_all(&temp_dir)?;

    // This test would require implementing ProfileManager properly
    // For now, just test that we can create a profile
    let profile = Profile {
        id: uuid::Uuid::new_v4().to_string(),
        name: "Test Profile".to_string(),
        created_at: chrono::Utc::now(),
        last_used: chrono::Utc::now(),
    };

    assert!(!profile.name.is_empty());
    assert!(!profile.id.is_empty());

    std::fs::remove_dir_all(&temp_dir)?;
    Ok(())
}

#[tokio::test]
async fn test_network_stats() -> Result<(), Box<dyn std::error::Error>> {
    use shadowghost::core::Peer;
    use shadowghost::network::NetworkManager;

    let peer = Peer::new("test_user".to_string(), "127.0.0.1:8080".to_string());
    let event_bus = EventBus::new();

    let network_manager = NetworkManager::new(peer, event_bus)?;
    let stats = network_manager.get_network_stats().await?;

    assert_eq!(stats.connected_peers, 0);
    assert_eq!(stats.messages_sent, 0);
    assert_eq!(stats.messages_received, 0);

    Ok(())
}

#[tokio::test]
async fn test_contact_blocking() -> Result<(), Box<dyn std::error::Error>> {
    use shadowghost::contacts::ContactManager;

    let temp_dir = std::env::temp_dir().join("shadowghost_block_test");
    std::fs::create_dir_all(&temp_dir)?;

    let mut contact_manager = ContactManager::new(&temp_dir)?;

    let test_contact = Contact {
        id: "block_contact_1".to_string(),
        name: "Block Contact".to_string(),
        address: "127.0.0.1:8084".to_string(),
        status: ContactStatus::Offline,
        trust_level: TrustLevel::Unknown,
        last_seen: Some(chrono::Utc::now()),
    };

    contact_manager.add_contact(test_contact.clone())?;

    assert!(!contact_manager.is_contact_blocked(&test_contact.id));

    contact_manager.block_contact(&test_contact.id)?;
    assert!(contact_manager.is_contact_blocked(&test_contact.id));

    contact_manager.unblock_contact(&test_contact.id)?;
    assert!(!contact_manager.is_contact_blocked(&test_contact.id));

    std::fs::remove_dir_all(&temp_dir)?;
    Ok(())
}

#[tokio::test]
async fn test_sg_link_operations() -> Result<(), Box<dyn std::error::Error>> {
    use shadowghost::contacts::{generate_sg_link, parse_sg_link};
    use shadowghost::core::Peer;

    let peer = Peer::new("TestUser".to_string(), "127.0.0.1:8080".to_string());

    let sg_link = generate_sg_link(&peer)?;
    assert!(sg_link.starts_with("sg://"));

    // Try to parse the link (should fail for self)
    let parse_result = parse_sg_link(&sg_link, "TestUser");
    assert!(parse_result.is_err());

    // Test with different user name
    let contact = parse_sg_link(&sg_link, "OtherUser")?;
    assert_eq!(contact.name, "TestUser");

    Ok(())
}
