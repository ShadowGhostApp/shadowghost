#[cfg(test)]
mod tests {
    use shadowghost::prelude::*;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::RwLock;
    use tokio::time::timeout;

    use common::{TestSetup, init_test_logging};

    #[tokio::test]
    async fn test_crypto_manager_creation() {
        let crypto = CryptoManager::new().unwrap();
        let public_key = crypto.get_public_key();
        assert_eq!(public_key.key_data.len(), 32);
        assert_eq!(public_key.key_bytes.len(), 32);
    }

    #[tokio::test]
    async fn test_crypto_encrypt_decrypt() {
        let crypto = CryptoManager::new().unwrap();
        let data = b"test message";

        let encrypted = crypto.encrypt(data).unwrap();
        let decrypted = crypto.decrypt(&encrypted).unwrap();

        assert_eq!(data, decrypted.as_slice());
    }

    #[tokio::test]
    async fn test_crypto_sign_verify() {
        let crypto = CryptoManager::new().unwrap();
        let data = b"test message";
        let public_key = crypto.get_public_key();

        let signature = crypto.sign_data(data).unwrap();
        let is_valid = crypto
            .verify_signature(data, &signature, &public_key)
            .unwrap();

        assert!(is_valid);
    }

    #[tokio::test]
    async fn test_crypto_shared_secret() {
        let crypto1 = CryptoManager::new().unwrap();
        let crypto2 = CryptoManager::new().unwrap();

        let pub_key1 = crypto1.get_public_key();
        let pub_key2 = crypto2.get_public_key();

        let secret1 = crypto1.derive_shared_secret(&pub_key2).unwrap();
        let secret2 = crypto2.derive_shared_secret(&pub_key1).unwrap();

        assert_eq!(secret1, secret2);
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
        let event_bus = EventBus::new();
        let mut receiver = event_bus.subscribe();

        let test_event = AppEvent::Network(NetworkEvent::ServerStarted { port: 8080 });
        event_bus.emit(test_event.clone());

        let received = timeout(Duration::from_millis(100), receiver.recv()).await;
        assert!(received.is_ok());
    }

    #[tokio::test]
    async fn test_config_manager() {
        let temp_dir = std::env::temp_dir().join("shadowghost_config_test");
        let config_path = temp_dir.join("test_config.toml");

        std::fs::create_dir_all(&temp_dir).unwrap();

        let mut config_manager = ConfigManager::new(&config_path).unwrap();
        let config = config_manager.get_config();

        assert_eq!(config.user.name, "user");
        assert_eq!(config.network.port, 8080);

        config_manager
            .set_user_name("new_user".to_string())
            .unwrap();
        let updated_config = config_manager.get_config();
        assert_eq!(updated_config.user.name, "new_user");

        std::fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[tokio::test]
    async fn test_config_validation() {
        let temp_dir = std::env::temp_dir().join("shadowghost_validation_test");
        let config_path = temp_dir.join("test_config.toml");

        std::fs::create_dir_all(&temp_dir).unwrap();

        let config_manager = ConfigManager::new(&config_path).unwrap();
        let issues = config_manager.validate_config().unwrap();

        assert!(issues.is_empty());

        std::fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[tokio::test]
    async fn test_storage_manager() {
        let temp_dir = std::env::temp_dir().join("shadowghost_storage_test");
        let mut config = AppConfig::default();
        config.storage.data_dir = temp_dir.to_string_lossy().to_string();

        let event_bus = EventBus::new();
        let mut storage_manager = StorageManager::new(config, event_bus).unwrap();
        storage_manager.initialize().await.unwrap();

        let mut contacts = HashMap::new();
        let contact = Contact {
            id: "test_id".to_string(),
            name: "test_user".to_string(),
            address: "127.0.0.1:8080".to_string(),
            status: ContactStatus::Online,
            trust_level: TrustLevel::Medium,
            last_seen: Some(chrono::Utc::now()),
        };
        contacts.insert("test_id".to_string(), contact);

        storage_manager.save_contacts(&contacts).await.unwrap();
        let loaded_contacts = storage_manager.load_contacts().await.unwrap();

        assert_eq!(loaded_contacts.len(), 1);
        assert_eq!(loaded_contacts.get("test_id").unwrap().name, "test_user");

        std::fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[tokio::test]
    async fn test_storage_chat_history() {
        let temp_dir = std::env::temp_dir().join("shadowghost_chat_test");
        let mut config = AppConfig::default();
        config.storage.data_dir = temp_dir.to_string_lossy().to_string();

        let event_bus = EventBus::new();
        let mut storage_manager = StorageManager::new(config, event_bus).unwrap();
        storage_manager.initialize().await.unwrap();

        let chat_message = ChatMessage {
            id: "msg1".to_string(),
            from: "alice".to_string(),
            to: "bob".to_string(),
            content: "hello".to_string(),
            msg_type: ChatMessageType::Text,
            timestamp: 1234567890,
            delivery_status: DeliveryStatus::Delivered,
        };

        let messages = vec![chat_message];
        storage_manager
            .save_chat_history("test_chat", &messages)
            .await
            .unwrap();

        let loaded_messages = storage_manager
            .load_chat_history("test_chat")
            .await
            .unwrap();
        assert_eq!(loaded_messages.len(), 1);
        assert_eq!(loaded_messages[0].content, "hello");

        std::fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[tokio::test]
    async fn test_contact_manager_sg_link() {
        let peer = Peer::new("test_user".to_string(), "127.0.0.1:8080".to_string());
        let crypto = Arc::new(RwLock::new(CryptoManager::new().unwrap()));
        let event_bus = EventBus::new();

        let contact_manager = ContactManager::new(peer, crypto, event_bus);

        let sg_link = contact_manager.generate_sg_link().await.unwrap();
        assert!(sg_link.starts_with("sg://"));


        let decoded_result = contact_manager.add_contact_by_sg_link(&sg_link).await;
        assert!(decoded_result.is_err());
        assert!(
            decoded_result
                .unwrap_err()
                .to_string()
                .contains("Cannot add yourself")
        );
    }

    #[tokio::test]
    async fn test_contact_manager_different_users() {
        let peer1 = Peer::new("user1".to_string(), "127.0.0.1:8080".to_string());
        let peer2 = Peer::new("user2".to_string(), "127.0.0.1:8081".to_string());

        let crypto1 = Arc::new(RwLock::new(CryptoManager::new().unwrap()));
        let crypto2 = Arc::new(RwLock::new(CryptoManager::new().unwrap()));

        let event_bus1 = EventBus::new();
        let event_bus2 = EventBus::new();

        let contact_manager1 = ContactManager::new(peer1, crypto1, event_bus1);
        let contact_manager2 = ContactManager::new(peer2, crypto2, event_bus2);

        let sg_link1 = contact_manager1.generate_sg_link().await.unwrap();
        let contact = contact_manager2
            .add_contact_by_sg_link(&sg_link1)
            .await
            .unwrap();

        assert_eq!(contact.name, "user1");
        assert_eq!(contact.address, "127.0.0.1:8080");
    }

    #[tokio::test]
    async fn test_network_manager_creation() {
        let peer = Peer::new("test_user".to_string(), "127.0.0.1:8080".to_string());
        let event_bus = EventBus::new();

        let network_manager = NetworkManager::new(peer.clone(), event_bus).unwrap();
        let retrieved_peer = network_manager.get_peer().await;

        assert_eq!(retrieved_peer.name, peer.name);
        assert_eq!(retrieved_peer.address, peer.address);
    }

    #[tokio::test]
    async fn test_network_manager_stats() {
        let peer = Peer::new("test_user".to_string(), "127.0.0.1:8080".to_string());
        let event_bus = EventBus::new();

        let network_manager = NetworkManager::new(peer, event_bus).unwrap();
        let stats = network_manager.get_network_stats().await.unwrap();

        assert_eq!(stats.messages_sent, 0);
        assert_eq!(stats.messages_received, 0);
        assert_eq!(stats.total_connections, 0);
    }

    #[tokio::test]
    async fn test_core_initialization() {
        init_test_logging();
        let setup = TestSetup::new("test_core_init").await.unwrap();

        assert!(setup.core.is_initialized());
        assert!(setup.core.get_peer_info().await.is_some());

        setup.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_core_server_lifecycle() {
        init_test_logging();
        let setup = TestSetup::new("test_server_lifecycle").await.unwrap();

        assert!(setup.core.is_server_started());

        setup.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_core_contact_management() {
        init_test_logging();
        let setup1 = TestSetup::new("user1").await.unwrap();
        let setup2 = TestSetup::new("user2").await.unwrap();

        let sg_link = setup1.core.generate_sg_link().await.unwrap();
        setup2.core.add_contact_by_sg_link(&sg_link).await.unwrap();

        let contacts = setup2.core.get_contacts().await.unwrap();
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].name, "user1");

        setup1.shutdown().await.unwrap();
        setup2.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_protocol_messages() {
        use shadowghost::protocol::*;

        let handshake = ProtocolMessage::create_handshake(
            "peer1".to_string(),
            "user1".to_string(),
            "127.0.0.1:8080".to_string(),
            vec![1, 2, 3, 4],
        );

        let bytes = handshake.to_bytes().unwrap();
        let reconstructed = ProtocolMessage::from_bytes(&bytes).unwrap();

        assert_eq!(handshake.header.sender_id, reconstructed.header.sender_id);
        assert_eq!(
            handshake.header.message_type,
            reconstructed.header.message_type
        );
    }

    #[tokio::test]
    async fn test_protocol_text_message() {
        use shadowghost::protocol::*;

        let text_msg = ProtocolMessage::create_text_message(
            "sender".to_string(),
            "recipient".to_string(),
            "Hello World".to_string(),
            "msg123".to_string(),
        );

        let bytes = text_msg.to_bytes().unwrap();
        let reconstructed = ProtocolMessage::from_bytes(&bytes).unwrap();

        match reconstructed.payload {
            MessagePayload::Text(text) => {
                assert_eq!(text.content, "Hello World");
                assert_eq!(text.message_id, "msg123");
            }
            _ => panic!("Expected text message"),
        }
    }

    #[tokio::test]
    async fn test_protocol_ping_pong() {
        use shadowghost::protocol::*;

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
    }

    #[tokio::test]
    async fn test_invalid_sg_link_formats() {
        let peer = Peer::new("test_user".to_string(), "127.0.0.1:8080".to_string());
        let crypto = Arc::new(RwLock::new(CryptoManager::new().unwrap()));
        let event_bus = EventBus::new();

        let contact_manager = ContactManager::new(peer, crypto, event_bus);

        let invalid_links = vec![
            "invalid://link",
            "sg://",
            "sg://invalid_base64!@#",
            "sg://dGVzdA==",
            "",
        ];

        for link in invalid_links {
            let result = contact_manager.add_contact_by_sg_link(link).await;
            assert!(result.is_err());
        }
    }

    #[tokio::test]
    async fn test_storage_validation() {
        let temp_dir = std::env::temp_dir().join("shadowghost_validation_test");
        let mut config = AppConfig::default();
        config.storage.data_dir = temp_dir.to_string_lossy().to_string();

        let event_bus = EventBus::new();
        let storage_manager = StorageManager::new(config, event_bus).unwrap();

        let contact_issues = storage_manager.validate_contacts().await.unwrap();
        let chat_issues = storage_manager.validate_chats().await.unwrap();

        assert!(contact_issues.is_empty());
        assert!(chat_issues.is_empty());

        std::fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[tokio::test]
    async fn test_storage_stats() {
        let temp_dir = std::env::temp_dir().join("shadowghost_stats_test");
        let mut config = AppConfig::default();
        config.storage.data_dir = temp_dir.to_string_lossy().to_string();

        let event_bus = EventBus::new();
        let storage_manager = StorageManager::new(config, event_bus).unwrap();

        let stats = storage_manager.get_storage_stats().await.unwrap();

        assert_eq!(stats.total_contacts, 0);
        assert_eq!(stats.total_chats, 0);
        assert_eq!(stats.total_messages, 0);

        std::fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[tokio::test]
    async fn test_core_name_update() {
        let test_id = uuid::Uuid::new_v4().to_string();
        let mut core = ShadowGhostCore::new_for_test(&test_id).unwrap();

        core.initialize(Some("original_name".to_string()))
            .await
            .unwrap();

        let peer_info = core.get_peer_info().await.unwrap();
        assert!(peer_info.contains("original_name"));

        core.update_user_name("new_name".to_string()).await.unwrap();

        let updated_peer_info = core.get_peer_info().await.unwrap();
        assert!(updated_peer_info.contains("new_name"));

        core.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_network_discovery() {
        use shadowghost::network_discovery::NetworkDiscovery;

        let result = timeout(Duration::from_secs(5), NetworkDiscovery::get_external_ip()).await;

        match result {
            Ok(Ok(ip)) => {
                assert!(!ip.to_string().is_empty());
            }
            Ok(Err(_)) => {}
            Err(_) => {}
        }
    }

    #[test]
    fn test_storage_stats_format_size() {
        use shadowghost::storage::StorageStats;

        let mut stats = StorageStats::default();

        stats.data_size_bytes = 512;
        assert_eq!(stats.format_size(), "512 B");

        stats.data_size_bytes = 1536;
        assert_eq!(stats.format_size(), "1.5 KB");

        stats.data_size_bytes = 1048576;
        assert_eq!(stats.format_size(), "1.0 MB");

        stats.data_size_bytes = 1073741824;
        assert_eq!(stats.format_size(), "1.0 GB");
    }

    #[test]
    fn test_peer_short_id() {
        let peer = Peer::new("test".to_string(), "127.0.0.1:8080".to_string());
        let short_id = peer.get_short_id();

        assert!(short_id.len() <= 8);
        assert!(!short_id.is_empty());
    }

    #[test]
    fn test_config_test_mode() {
        let temp_dir = std::env::temp_dir().join("shadowghost_test_mode");
        let config_path = temp_dir.join("test_config.toml");

        std::fs::create_dir_all(&temp_dir).unwrap();

        let mut config_manager = ConfigManager::new(&config_path).unwrap();
        config_manager.enable_test_mode().unwrap();

        let config = config_manager.get_config();
        assert!(config.network.test_mode);
        assert!(!config.network.auto_detect_external_ip);
        assert!(!config.storage.enable_backup);

        std::fs::remove_dir_all(&temp_dir).unwrap();
    }
}
