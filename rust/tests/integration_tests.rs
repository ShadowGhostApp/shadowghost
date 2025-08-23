use shadowghost::prelude::*;
use tokio_test;

#[tokio::test]
async fn test_core_initialization() {
    let mut core = ShadowGhostCore::new_for_test("init_test").unwrap();
    
    assert!(!core.is_initialized());
    
    let result = core.initialize(Some("TestUser".to_string())).await;
    assert!(result.is_ok());
    assert!(core.is_initialized());
}

#[tokio::test]
async fn test_contact_management() {
    let mut core = ShadowGhostCore::new_for_test("contact_test").unwrap();
    core.initialize(Some("TestUser".to_string())).await.unwrap();

    let contact = Contact {
        id: "test_contact_1".to_string(),
        name: "Test Contact".to_string(),
        address: "127.0.0.1:8081".to_string(),
        status: ContactStatus::Offline,
        trust_level: TrustLevel::Unknown,
        last_seen: Some(chrono::Utc::now()),
    };

    let add_result = core.add_contact_manual(contact.clone()).await;
    assert!(add_result.is_ok());

    let contacts = core.get_contacts().await.unwrap();
    assert_eq!(contacts.len(), 1);
    assert_eq!(contacts[0].name, "Test Contact");

    let trust_result = core.update_contact_trust_level(&contact.id, TrustLevel::Trusted).await;
    assert!(trust_result.is_ok());

    let remove_result = core.remove_contact_by_id(&contact.id).await;
    assert!(remove_result.is_ok());

    let contacts_after_remove = core.get_contacts().await.unwrap();
    assert_eq!(contacts_after_remove.len(), 0);
}

#[tokio::test]
async fn test_message_operations() {
    let mut core = ShadowGhostCore::new_for_test("message_test").unwrap();
    core.initialize(Some("TestUser".to_string())).await.unwrap();

    let contact = Contact {
        id: "msg_contact_1".to_string(),
        name: "Message Contact".to_string(),
        address: "127.0.0.1:8082".to_string(),
        status: ContactStatus::Online,
        trust_level: TrustLevel::Trusted,
        last_seen: Some(chrono::Utc::now()),
    };

    core.add_contact_manual(contact.clone()).await.unwrap();

    let send_result = core.send_message("Message Contact", "Hello, World!").await;
    assert!(send_result.is_ok());

    let messages = core.get_chat_messages("Message Contact").await.unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].content, "Hello, World!");

    let mark_result = core.mark_messages_as_read("Message Contact").await;
    assert!(mark_result.is_ok());
}

#[tokio::test]
async fn test_network_operations() {
    let mut core = ShadowGhostCore::new_for_test("network_test").unwrap();
    core.initialize(Some("NetworkUser".to_string())).await.unwrap();

    let server_start = core.start_server().await;
    assert!(server_start.is_ok());
    assert!(core.is_server_started());

    let stats = core.get_network_stats().await.unwrap();
    assert_eq!(stats.connected_peers, 0);

    let server_stop = core.stop_server().await;
    assert!(server_stop.is_ok());
}

#[tokio::test]
async fn test_storage_operations() {
    let storage_manager = StorageManager::new_with_path("./test_storage".to_string()).unwrap();
    let mut storage = storage_manager;

    let init_result = storage.initialize().await;
    assert!(init_result.is_ok());

    let test_contact = Contact {
        id: "storage_contact_1".to_string(),
        name: "Storage Contact".to_string(),
        address: "127.0.0.1:8083".to_string(),
        status: ContactStatus::Offline,
        trust_level: TrustLevel::Unknown,
        last_seen: Some(chrono::Utc::now()),
    };

    let save_result = storage.save_contact(&test_contact).await;
    assert!(save_result.is_ok());

    let retrieved_contact = storage.get_contact(&test_contact.id).await.unwrap();
    assert!(retrieved_contact.is_some());
    assert_eq!(retrieved_contact.unwrap().name, "Storage Contact");

    let delete_result = storage.delete_contact(&test_contact.id).await;
    assert!(delete_result.is_ok());

    let deleted_contact = storage.get_contact(&test_contact.id).await.unwrap();
    assert!(deleted_contact.is_none());
}

#[tokio::test]
async fn test_crypto_operations() {
    let crypto_manager = CryptoManager::new().unwrap();

    let public_key = crypto_manager.get_public_key().unwrap();
    assert!(!public_key.key_data.is_empty());

    let test_data = b"Hello, cryptographic world!";
    let encrypted = crypto_manager.encrypt_message(test_data, &public_key).unwrap();
    assert_ne!(encrypted.data, test_data);

    let decrypted = crypto_manager.decrypt_message(&encrypted).unwrap();
    assert_eq!(decrypted, test_data);

    let signature = crypto_manager.sign_data(test_data).unwrap();
    let verification = crypto_manager.verify_signature(test_data, &signature, &public_key).unwrap();
    assert!(verification);

    let session_key = crypto_manager.generate_session_key();
    let (encrypted_data, nonce) = crypto_manager.encrypt_with_session_key(test_data, &session_key).unwrap();
    let decrypted_data = crypto_manager.decrypt_with_session_key(&encrypted_data, &nonce, &session_key).unwrap();
    assert_eq!(decrypted_data, test_data);
}

#[tokio::test]
async fn test_contact_blocking() {
    use shadowghost::contacts::ContactManager;
    use shadowghost::core::peer::Peer;
    
    let peer = Peer::new("TestUser".to_string(), "127.0.0.1:8080".to_string());
    let mut contact_manager = ContactManager::new(peer);

    let test_contact = Contact {
        id: "block_contact_1".to_string(),
        name: "Block Contact".to_string(),
        address: "127.0.0.1:8084".to_string(),
        status: ContactStatus::Offline,
        trust_level: TrustLevel::Unknown,
        last_seen: Some(chrono::Utc::now()),
    };

    contact_manager.add_contact(test_contact.clone()).unwrap();

    assert!(!contact_manager.is_contact_blocked(&test_contact.id));

    let block_result = contact_manager.block_contact(&test_contact.id);
    assert!(block_result.is_ok());
    assert!(contact_manager.is_contact_blocked(&test_contact.id));

    let unblock_result = contact_manager.unblock_contact(&test_contact.id);
    assert!(unblock_result.is_ok());
    assert!(!contact_manager.is_contact_blocked(&test_contact.id));

    let blocked_contacts = contact_manager.get_blocked_contacts();
    assert_eq!(blocked_contacts.len(), 0);
}

#[tokio::test]
async fn test_network_discovery() {
    use shadowghost::network::NetworkDiscovery;
    
    let mut discovery = NetworkDiscovery::new(
        8080,
        "test_peer_1".to_string(),
        "Test Peer".to_string(),
        vec![1, 2, 3, 4],
    );

    assert!(!discovery.is_running());

    let start_result = discovery.start_discovery().await;
    assert!(start_result.is_ok());

    let announce_result = discovery.announce_presence().await;
    assert!(announce_result.is_ok());

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let peer_count = discovery.get_peer_count().await;
    
    discovery.cleanup_old_peers(1).await;

    discovery.stop_discovery().await;
    assert!(!discovery.is_running());
}

#[tokio::test]
async fn test_sg_link_operations() {
    let mut core = ShadowGhostCore::new_for_test("sg_link_test").unwrap();
    core.initialize(Some("LinkUser".to_string())).await.unwrap();

    let sg_link = core.generate_sg_link().await.unwrap();
    assert!(sg_link.starts_with("sg://"));

    let add_result = core.add_contact_by_sg_link(&sg_link).await;
    assert!(add_result.is_err());
}

#[tokio::test]
async fn test_system_health() {
    let mut core = ShadowGhostCore::new_for_test("health_test").unwrap();
    core.initialize(Some("HealthUser".to_string())).await.unwrap();

    let memory_usage = core.get_memory_usage();
    assert!(memory_usage >= 0);

    let memory_info = core.get_system_memory_info().await.unwrap();
    assert!(memory_info.total_memory > 0);
    assert!(memory_info.available_memory > 0);

    let error_stats = core.get_error_statistics().await.unwrap();
    assert!(error_stats.total_errors >= 0);
}

#[tokio::test]
async fn test_message_delivery_status() {
    let mut core = ShadowGhostCore::new_for_test("delivery_test").unwrap();
    core.initialize(Some("DeliveryUser".to_string())).await.unwrap();

    let contact = Contact {
        id: "delivery_contact".to_string(),
        name: "Delivery Contact".to_string(),
        address: "127.0.0.1:8085".to_string(),
        status: ContactStatus::Online,
        trust_level: TrustLevel::Trusted,
        last_seen: Some(chrono::Utc::now()),
    };

    core.add_contact_manual(contact).await.unwrap();

    core.send_message("Delivery Contact", "Test message").await.unwrap();

    let messages = core.get_chat_messages("Delivery Contact").await.unwrap();
    assert_eq!(messages.len(), 1);

    let message = &messages[0];
    let status = core.get_message_delivery_status(&message.id).await.unwrap();
    assert!(matches!(status, DeliveryStatus::Pending | DeliveryStatus::Sent));
}

#[tokio::test]
async fn test_storage_optimization() {
    let mut storage = StorageManager::new_with_path("./test_optimization".to_string()).unwrap();
    storage.initialize().await.unwrap();

    let message = ChatMessage {
        id: "opt_msg_1".to_string(),
        from: "user1".to_string(),
        to: "user2".to_string(),
        content: "Optimization test message".to_string(),
        msg_type: ChatMessageType::Text,
        timestamp: chrono::Utc::now().timestamp() as u64,
        delivery_status: DeliveryStatus::Delivered,
    };

    storage.save_message("test_chat", &message).await.unwrap();

    let optimization_result = storage.optimize_storage().await.unwrap();
    assert!(optimization_result.original_size_bytes >= optimization_result.optimized_size_bytes);

    let cleanup_count = storage.cleanup_old_messages(0).await.unwrap();
    assert!(cleanup_count >= 0);

    let health = storage.get_storage_health().await.unwrap();
    assert!(health.total_size_bytes >= 0);
    assert!(health.fragmentation_percent >= 0.0);
}

#[tokio::test]
async fn test_performance_under_load() {
    let mut core = ShadowGhostCore::new_for_test("load_test").unwrap();
    core.initialize(Some("LoadUser".to_string())).await.unwrap();

    let mut contacts = Vec::new();
    for i in 0..10 {
        let contact = Contact {
            id: format!("load_contact_{}", i),
            name: format!("Load Contact {}", i),
            address: format!("127.0.0.1:808{}", i),
            status: ContactStatus::Offline,
            trust_level: TrustLevel::Unknown,
            last_seen: Some(chrono::Utc::now()),
        };
        contacts.push(contact);
    }

    for contact in &contacts {
        core.add_contact_manual(contact.clone()).await.unwrap();
    }

    let retrieved_contacts = core.get_contacts().await.unwrap();
    assert_eq!(retrieved_contacts.len(), 10);

    for (i, contact) in contacts.iter().enumerate() {
        let send_result = core.send_message(&contact.name, &format!("Message {}", i)).await;
        assert!(send_result.is_ok());
    }

    for contact in &contacts {
        let messages = core.get_chat_messages(&contact.name).await.unwrap();
        assert_eq!(messages.len(), 1);
    }
}