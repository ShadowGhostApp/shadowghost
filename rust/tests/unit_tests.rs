// tests/unit_tests.rs
use shadowghost::chats::Manager as ChatManager;
use shadowghost::contacts::ContactManager;
use shadowghost::core::{Engine, Peer, Profile};
use shadowghost::crypto::CryptoManager;
use shadowghost::events::EventBus;
use shadowghost::network::{
    ChatMessage, ChatMessageType, Contact, ContactStatus, DeliveryStatus, NetworkManager,
    TrustLevel,
};
use shadowghost::storage::StorageManager;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_crypto_manager_creation() -> Result<(), Box<dyn std::error::Error>> {
    let crypto = CryptoManager::new()?;
    let public_key = crypto.get_public_key();
    assert!(!public_key.key_data.is_empty());
    assert_eq!(public_key.algorithm, "Ed25519");
    Ok(())
}

#[tokio::test]
async fn test_crypto_encrypt_decrypt() -> Result<(), Box<dyn std::error::Error>> {
    let crypto = CryptoManager::new()?;
    let data = b"test message";

    let encrypted = crypto.encrypt(data)?;
    let decrypted = crypto.decrypt(&encrypted)?;

    assert_eq!(data, decrypted.as_slice());
    Ok(())
}

#[tokio::test]
async fn test_crypto_sign_verify() -> Result<(), Box<dyn std::error::Error>> {
    let crypto = CryptoManager::new()?;
    let data = b"test message";
    let public_key = crypto.get_public_key();

    let signature = crypto.sign_data(data)?;
    let is_valid = crypto.verify_signature(data, &signature, &public_key)?;

    assert!(is_valid);
    Ok(())
}

#[tokio::test]
async fn test_crypto_shared_secret() -> Result<(), Box<dyn std::error::Error>> {
    let crypto1 = CryptoManager::new()?;
    let crypto2 = CryptoManager::new()?;

    let pub_key1 = crypto1.get_public_key();
    let pub_key2 = crypto2.get_public_key();

    let secret1 = crypto1.derive_shared_secret(&pub_key2)?;
    let secret2 = crypto2.derive_shared_secret(&pub_key1)?;

    assert_eq!(secret1, secret2);
    Ok(())
}

#[tokio::test]
async fn test_peer_creation() {
    let peer = Peer::new("test_user".to_string(), "127.0.0.1:8080".to_string());

    assert_eq!(peer.name, "test_user");
    assert_eq!(peer.address, "127.0.0.1");
    assert_eq!(peer.port, 8080);
    assert!(!peer.id.is_empty());
}

#[tokio::test]
async fn test_peer_entropy() {
    let peer1 = Peer::new_with_entropy("user".to_string(), "127.0.0.1:8080".to_string());
    let peer2 = Peer::new_with_entropy("user".to_string(), "127.0.0.1:8080".to_string());

    assert_ne!(peer1.id, peer2.id);
}

#[tokio::test]
async fn test_event_bus() {
    use shadowghost::events::{AppEvent, NetworkEvent};

    let event_bus = EventBus::new();
    let mut receiver = event_bus.subscribe();

    let test_event = AppEvent::Network(NetworkEvent::ServerStarted { port: 8080 });
    event_bus.emit(test_event.clone());

    let received = tokio::time::timeout(Duration::from_millis(100), receiver.recv()).await;
    assert!(received.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_storage_manager() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = std::env::temp_dir().join("shadowghost_storage_test");
    std::fs::create_dir_all(&temp_dir)?;

    let event_bus = EventBus::new();
    let storage_manager = StorageManager::new(&temp_dir, event_bus)?;
    storage_manager.initialize().await?;

    let contact = Contact {
        id: "test_id".to_string(),
        name: "test_user".to_string(),
        address: "127.0.0.1:8080".to_string(),
        status: ContactStatus::Online,
        trust_level: TrustLevel::Medium,
        last_seen: Some(chrono::Utc::now()),
    };

    storage_manager.save_contact(&contact).await?;
    let loaded_contacts = storage_manager.get_contacts().await?;

    assert_eq!(loaded_contacts.len(), 1);
    assert_eq!(loaded_contacts[0].name, "test_user");

    std::fs::remove_dir_all(&temp_dir)?;
    Ok(())
}

#[tokio::test]
async fn test_storage_chat_history() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = std::env::temp_dir().join("shadowghost_chat_test");
    std::fs::create_dir_all(&temp_dir)?;

    let event_bus = EventBus::new();
    let storage_manager = StorageManager::new(&temp_dir, event_bus)?;
    storage_manager.initialize().await?;

    let chat_message = ChatMessage {
        id: "msg1".to_string(),
        from: "alice".to_string(),
        to: "bob".to_string(),
        content: "hello".to_string(),
        msg_type: ChatMessageType::Text,
        timestamp: chrono::Utc::now().timestamp() as u64,
        delivery_status: DeliveryStatus::Delivered,
    };

    storage_manager
        .save_message("test_chat", &chat_message)
        .await?;

    let loaded_messages = storage_manager.get_messages("test_chat").await?;
    assert_eq!(loaded_messages.len(), 1);
    assert_eq!(loaded_messages[0].content, "hello");

    std::fs::remove_dir_all(&temp_dir)?;
    Ok(())
}

#[tokio::test]
async fn test_contact_manager_sg_link() -> Result<(), Box<dyn std::error::Error>> {
    use shadowghost::contacts::{generate_sg_link, parse_sg_link};

    let peer = Peer::new("test_user".to_string(), "127.0.0.1:8080".to_string());

    let sg_link = generate_sg_link(&peer)?;
    assert!(sg_link.starts_with("sg://"));

    // Test that we cannot add ourselves
    let decoded_result = parse_sg_link(&sg_link, "test_user");
    assert!(decoded_result.is_err());
    assert!(decoded_result
        .unwrap_err()
        .to_string()
        .contains("Cannot add yourself"));

    Ok(())
}

#[tokio::test]
async fn test_contact_manager_different_users() -> Result<(), Box<dyn std::error::Error>> {
    use shadowghost::contacts::{generate_sg_link, parse_sg_link};

    let peer1 = Peer::new("user1".to_string(), "127.0.0.1:8080".to_string());
    let peer2 = Peer::new("user2".to_string(), "127.0.0.1:8081".to_string());

    let sg_link1 = generate_sg_link(&peer1)?;
    let contact = parse_sg_link(&sg_link1, "user2")?;

    assert_eq!(contact.name, "user1");
    assert_eq!(contact.address, "127.0.0.1:8080");

    Ok(())
}

#[tokio::test]
async fn test_network_manager_creation() -> Result<(), Box<dyn std::error::Error>> {
    let peer = Peer::new("test_user".to_string(), "127.0.0.1:8080".to_string());
    let event_bus = EventBus::new();

    let network_manager = NetworkManager::new(peer.clone(), event_bus)?;
    let retrieved_peer = network_manager.get_peer().await;

    assert_eq!(retrieved_peer.name, peer.name);
    assert_eq!(retrieved_peer.address, peer.address);

    Ok(())
}

#[tokio::test]
async fn test_network_manager_stats() -> Result<(), Box<dyn std::error::Error>> {
    let peer = Peer::new("test_user".to_string(), "127.0.0.1:8080".to_string());
    let event_bus = EventBus::new();

    let network_manager = NetworkManager::new(peer, event_bus)?;
    let stats = network_manager.get_network_stats().await?;

    assert_eq!(stats.messages_sent, 0);
    assert_eq!(stats.messages_received, 0);
    assert_eq!(stats.connected_peers, 0);

    Ok(())
}

#[tokio::test]
async fn test_protocol_messages() -> Result<(), Box<dyn std::error::Error>> {
    use shadowghost::network::{MessageType, ProtocolMessage};

    let handshake = ProtocolMessage::create_handshake(
        "peer1".to_string(),
        "user1".to_string(),
        "127.0.0.1:8080".to_string(),
        vec![1, 2, 3, 4],
    );

    let bytes = handshake.to_bytes()?;
    let reconstructed = ProtocolMessage::from_bytes(&bytes)?;

    assert_eq!(handshake.header.sender_id, reconstructed.header.sender_id);
    assert_eq!(
        handshake.header.message_type,
        reconstructed.header.message_type
    );

    Ok(())
}

#[tokio::test]
async fn test_protocol_text_message() -> Result<(), Box<dyn std::error::Error>> {
    use shadowghost::network::{MessagePayload, ProtocolMessage};

    let text_msg = ProtocolMessage::create_text_message(
        "sender".to_string(),
        "recipient".to_string(),
        "Hello World".to_string(),
        "msg123".to_string(),
    );

    let bytes = text_msg.to_bytes()?;
    let reconstructed = ProtocolMessage::from_bytes(&bytes)?;

    match reconstructed.payload {
        MessagePayload::Text(text) => {
            assert_eq!(text.content, "Hello World");
            assert_eq!(text.message_id, "msg123");
        }
        _ => panic!("Expected text message"),
    }

    Ok(())
}

#[tokio::test]
async fn test_protocol_ping_pong() -> Result<(), Box<dyn std::error::Error>> {
    use shadowghost::network::{MessagePayload, ProtocolMessage};

    let ping = ProtocolMessage::create_ping("sender".to_string(), "recipient".to_string());
    let timestamp = match &ping.payload {
        MessagePayload::Ping(p) => p.timestamp,
        _ => panic!("Expected ping message"),
    };

    let pong =
        ProtocolMessage::create_pong("recipient".to_string(), "sender".to_string(), timestamp);

    match &pong.payload {
        MessagePayload::Pong(p) => {
            assert_eq!(p.original_timestamp, timestamp);
            assert!(p.response_timestamp >= timestamp);
        }
        _ => panic!("Expected pong message"),
    }

    Ok(())
}

#[tokio::test]
async fn test_invalid_sg_link_formats() -> Result<(), Box<dyn std::error::Error>> {
    use shadowghost::contacts::parse_sg_link;

    let invalid_links = vec![
        "invalid://link",
        "sg://",
        "sg://invalid_base64!@#",
        "sg://dGVzdA==",
        "",
    ];

    for link in invalid_links {
        let result = parse_sg_link(link, "test_user");
        assert!(
            result.is_err(),
            "Invalid link '{}' should be rejected",
            link
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_storage_validation() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = std::env::temp_dir().join("shadowghost_validation_test");
    std::fs::create_dir_all(&temp_dir)?;

    let event_bus = EventBus::new();
    let storage_manager = StorageManager::new(&temp_dir, event_bus)?;
    storage_manager.initialize().await?;

    let contact_issues = storage_manager.validate_contacts().await?;
    let chat_issues = storage_manager.validate_chats().await?;

    assert!(contact_issues.is_empty());
    assert!(chat_issues.is_empty());

    std::fs::remove_dir_all(&temp_dir)?;
    Ok(())
}

#[tokio::test]
async fn test_storage_stats() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = std::env::temp_dir().join("shadowghost_stats_test");
    std::fs::create_dir_all(&temp_dir)?;

    let event_bus = EventBus::new();
    let storage_manager = StorageManager::new(&temp_dir, event_bus)?;
    storage_manager.initialize().await?;

    let stats = storage_manager.get_stats().await?;

    assert_eq!(stats.contact_count, 0);
    assert_eq!(stats.chat_count, 0);
    assert_eq!(stats.message_count, 0);

    std::fs::remove_dir_all(&temp_dir)?;
    Ok(())
}

#[tokio::test]
async fn test_peer_short_id() {
    let peer = Peer::new("test".to_string(), "127.0.0.1:8080".to_string());
    let short_id = peer.get_short_id();

    assert!(short_id.len() <= 8);
    assert!(!short_id.is_empty());
}

#[tokio::test]
async fn test_chat_manager_basic_operations() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = std::env::temp_dir().join("shadowghost_chat_manager_test");
    std::fs::create_dir_all(&temp_dir)?;

    let event_bus = EventBus::new();
    let storage_manager = Arc::new(RwLock::new(StorageManager::new(
        &temp_dir,
        event_bus.clone(),
    )?));

    let chat_manager = ChatManager::new(storage_manager, event_bus)?;

    // Create a chat
    let chat = chat_manager
        .create_chat("Test Chat".to_string(), false)
        .await?;
    assert_eq!(chat.name, "Test Chat");
    assert!(!chat.is_group);

    // Get all chats
    let chats = chat_manager.get_all_chats().await;
    assert_eq!(chats.len(), 1);

    std::fs::remove_dir_all(&temp_dir)?;
    Ok(())
}

#[tokio::test]
async fn test_contact_stats() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = std::env::temp_dir().join("shadowghost_contact_stats_test");
    std::fs::create_dir_all(&temp_dir)?;

    let mut contact_manager = ContactManager::new(&temp_dir)?;

    let contact1 = Contact {
        id: "1".to_string(),
        name: "User1".to_string(),
        address: "127.0.0.1:8080".to_string(),
        status: ContactStatus::Online,
        trust_level: TrustLevel::Trusted,
        last_seen: Some(chrono::Utc::now()),
    };

    let contact2 = Contact {
        id: "2".to_string(),
        name: "User2".to_string(),
        address: "127.0.0.1:8081".to_string(),
        status: ContactStatus::Offline,
        trust_level: TrustLevel::Unknown,
        last_seen: Some(chrono::Utc::now()),
    };

    contact_manager.add_contact(contact1)?;
    contact_manager.add_contact(contact2)?;
    contact_manager.block_contact("2")?;

    let stats = contact_manager.get_contact_stats();
    assert_eq!(stats.total_contacts, 2);
    assert_eq!(stats.online_contacts, 1);
    assert_eq!(stats.trusted_contacts, 1);
    assert_eq!(stats.blocked_contacts, 1);

    std::fs::remove_dir_all(&temp_dir)?;
    Ok(())
}
