#[cfg(test)]
mod stress_tests {
    use shadowghost::prelude::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    use tokio::sync::{RwLock, Semaphore};
    use tokio::time::sleep;

    mod common;
    use common::{init_test_logging, TestSetup};

    #[tokio::test]
    async fn stress_test_crypto_operations() {
        let num_operations = 1000;
        let start_time = Instant::now();

        let mut handles = vec![];

        for i in 0..num_operations {
            let handle = tokio::spawn(async move {
                let crypto = CryptoManager::new().unwrap();
                let data = format!("test message {}", i).as_bytes().to_vec();

                let encrypted = crypto.encrypt(&data).unwrap();
                let decrypted = crypto.decrypt(&encrypted).unwrap();

                assert_eq!(data, decrypted);

                let public_key = crypto.get_public_key();
                let signature = crypto.sign_data(&data).unwrap();
                let is_valid = crypto
                    .verify_signature(&data, &signature, &public_key)
                    .unwrap();

                assert!(is_valid);
                i
            });
            handles.push(handle);
        }

        let mut results = vec![];
        for handle in handles {
            results.push(handle.await.unwrap());
        }

        let elapsed = start_time.elapsed();
        println!(
            "Completed {} crypto operations in {:?}",
            num_operations, elapsed
        );
        println!("Average: {:?} per operation", elapsed / num_operations);

        assert_eq!(results.len(), num_operations);
    }

    #[tokio::test]
    async fn stress_test_contact_generation() {
        let num_contacts = 500;
        let start_time = Instant::now();

        let semaphore = Arc::new(Semaphore::new(50));
        let mut handles = vec![];

        for i in 0..num_contacts {
            let sem = semaphore.clone();
            let handle = tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();

                let peer = Peer::new(
                    format!("user_{}", i),
                    format!("127.0.0.1:{}", 8000 + (i % 1000)),
                );
                let crypto = Arc::new(RwLock::new(CryptoManager::new().unwrap()));
                let event_bus = EventBus::new();

                let contact_manager = ContactManager::new(peer, crypto, event_bus);
                let sg_link = contact_manager.generate_sg_link().await.unwrap();

                assert!(sg_link.starts_with("sg://"));
                i
            });
            handles.push(handle);
        }

        let mut results = vec![];
        for handle in handles {
            results.push(handle.await.unwrap());
        }

        let elapsed = start_time.elapsed();
        println!("Generated {} SG links in {:?}", num_contacts, elapsed);
        println!("Average: {:?} per link", elapsed / num_contacts);

        assert_eq!(results.len(), num_contacts);
    }

    #[tokio::test]
    async fn stress_test_storage_operations() {
        let num_operations = 200;
        let temp_dir = std::env::temp_dir().join("shadowghost_stress_storage");

        let mut config = AppConfig::default();
        config.storage.data_dir = temp_dir.clone();

        let event_bus = EventBus::new();
        let storage_manager = Arc::new(StorageManager::new(config, event_bus).unwrap());

        let start_time = Instant::now();
        let semaphore = Arc::new(Semaphore::new(20));
        let mut handles = vec![];

        for i in 0..num_operations {
            let storage = storage_manager.clone();
            let sem = semaphore.clone();

            let handle = tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();

                let mut contacts = HashMap::new();
                for j in 0..10 {
                    let contact = Contact {
                        id: format!("contact_{}_{}", i, j),
                        name: format!("user_{}_{}", i, j),
                        address: format!("127.0.0.1:{}", 8000 + j),
                        status: ContactStatus::Online,
                        trust_level: TrustLevel::Medium,
                        last_seen: 1234567890 + i as u64,
                    };
                    contacts.insert(format!("contact_{}_{}", i, j), contact);
                }

                storage.save_contacts(&contacts).await.unwrap();
                let loaded = storage.load_contacts().await.unwrap();

                assert_eq!(loaded.len(), contacts.len());

                let messages: Vec<ChatMessage> = (0..5)
                    .map(|k| ChatMessage {
                        id: format!("msg_{}_{}_{}", i, j, k),
                        from: format!("sender_{}", i),
                        to: format!("recipient_{}", i),
                        content: format!("Message {} from operation {}", k, i),
                        msg_type: ChatMessageType::Text,
                        timestamp: 1234567890 + (i * 10 + k) as u64,
                        delivery_status: DeliveryStatus::Delivered,
                    })
                    .collect();

                let chat_id = format!("stress_chat_{}", i);
                storage
                    .save_chat_history(&chat_id, &messages)
                    .await
                    .unwrap();
                let loaded_messages = storage.load_chat_history(&chat_id).await.unwrap();

                assert_eq!(loaded_messages.len(), messages.len());
                i
            });
            handles.push(handle);
        }

        let mut results = vec![];
        for handle in handles {
            results.push(handle.await.unwrap());
        }

        let elapsed = start_time.elapsed();
        println!(
            "Completed {} storage operations in {:?}",
            num_operations, elapsed
        );
        println!("Average: {:?} per operation", elapsed / num_operations);

        assert_eq!(results.len(), num_operations);

        std::fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[tokio::test]
    async fn stress_test_event_system() {
        let num_events = 10000;
        let num_subscribers = 10;

        let event_bus = Arc::new(EventBus::new());
        let mut receivers = vec![];

        for _ in 0..num_subscribers {
            receivers.push(event_bus.subscribe());
        }

        let start_time = Instant::now();

        let sender_bus = event_bus.clone();
        let sender_handle = tokio::spawn(async move {
            for i in 0..num_events {
                sender_bus.emit_network(NetworkEvent::ServerStarted {
                    port: 8000 + (i % 1000) as u16,
                });
                if i % 100 == 0 {
                    sleep(Duration::from_millis(1)).await;
                }
            }
        });

        let mut receiver_handles = vec![];
        for (idx, mut receiver) in receivers.into_iter().enumerate() {
            let handle = tokio::spawn(async move {
                let mut count = 0;
                let timeout_duration = Duration::from_secs(10);

                while count < num_events {
                    match tokio::time::timeout(timeout_duration, receiver.recv()).await {
                        Ok(Ok(_)) => count += 1,
                        Ok(Err(_)) => break,
                        Err(_) => break,
                    }
                }
                (idx, count)
            });
            receiver_handles.push(handle);
        }

        sender_handle.await.unwrap();

        let mut total_received = 0;
        for handle in receiver_handles {
            let (idx, count) = handle.await.unwrap();
            total_received += count;
            println!("Receiver {} got {} events", idx, count);
        }

        let elapsed = start_time.elapsed();
        println!("Event stress test completed in {:?}", elapsed);
        println!("Total events sent: {}", num_events);
        println!("Total events received: {}", total_received);
        println!("Expected total: {}", num_events * num_subscribers);

        assert!(total_received >= num_events * num_subscribers / 2);
    }

    #[tokio::test]
    async fn stress_test_protocol_serialization() {
        use shadowghost::protocol::*;

        let num_operations = 5000;
        let start_time = Instant::now();

        let mut handles = vec![];

        for i in 0..num_operations {
            let handle = tokio::spawn(async move {
                let message_types = vec![
                    ProtocolMessage::create_handshake(
                        format!("peer_{}", i),
                        format!("user_{}", i),
                        format!("127.0.0.1:{}", 8000 + (i % 1000)),
                        vec![i as u8; 32],
                    ),
                    ProtocolMessage::create_text_message(
                        format!("sender_{}", i),
                        format!("recipient_{}", i),
                        format!(
                            "Message number {} with some content to test serialization performance",
                            i
                        ),
                        format!("msg_{}", i),
                    ),
                    ProtocolMessage::create_ping(format!("pinger_{}", i), format!("pingee_{}", i)),
                ];

                let mut total_bytes = 0;
                for msg in message_types {
                    let serialized = msg.to_bytes().unwrap();
                    total_bytes += serialized.len();

                    let deserialized = ProtocolMessage::from_bytes(&serialized).unwrap();
                    assert_eq!(msg.header.sender_id, deserialized.header.sender_id);
                }

                (i, total_bytes)
            });
            handles.push(handle);
        }

        let mut total_bytes = 0;
        for handle in handles {
            let (_, bytes) = handle.await.unwrap();
            total_bytes += bytes;
        }

        let elapsed = start_time.elapsed();
        println!(
            "Protocol serialization stress test completed in {:?}",
            elapsed
        );
        println!("Operations: {}", num_operations);
        println!("Total bytes processed: {}", total_bytes);
        println!("Average: {:?} per operation", elapsed / num_operations);
    }

    #[tokio::test]
    async fn stress_test_multiple_cores() {
        init_test_logging();
        let num_cores = 10;
        let start_time = Instant::now();

        let semaphore = Arc::new(Semaphore::new(5));
        let mut handles = vec![];

        for i in 0..num_cores {
            let sem = semaphore.clone();
            let handle = tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();

                let test_name = format!("stress_user_{}", i);
                let setup = TestSetup::new(&test_name).await.unwrap();

                sleep(Duration::from_millis(50)).await;

                let sg_link = setup.core.generate_sg_link().await.unwrap();
                assert!(sg_link.starts_with("sg://"));

                let peer_info = setup.core.get_peer_info().await.unwrap();
                assert!(peer_info.contains(&test_name));

                let stats = setup.core.get_network_stats().await.unwrap();
                assert_eq!(stats.messages_sent, 0);

                setup.shutdown().await.unwrap();
                i
            });
            handles.push(handle);
        }

        let mut results = vec![];
        for handle in handles {
            results.push(handle.await.unwrap());
        }

        let elapsed = start_time.elapsed();
        println!("Multiple cores stress test completed in {:?}", elapsed);
        println!("Created and managed {} cores", num_cores);
        println!("Average: {:?} per core", elapsed / num_cores);

        assert_eq!(results.len(), num_cores);
    }

    #[tokio::test]
    async fn stress_test_config_operations() {
        let num_operations = 1000;
        let temp_base_dir = std::env::temp_dir().join("shadowghost_config_stress");

        std::fs::create_dir_all(&temp_base_dir).unwrap();

        let start_time = Instant::now();
        let semaphore = Arc::new(Semaphore::new(100));
        let mut handles = vec![];

        for i in 0..num_operations {
            let base_dir = temp_base_dir.clone();
            let sem = semaphore.clone();

            let handle = tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();

                let config_path = base_dir.join(format!("config_{}.toml", i));
                let mut config_manager = ConfigManager::new(&config_path).unwrap();

                config_manager
                    .set_user_name(format!("stress_user_{}", i))
                    .unwrap();
                config_manager
                    .set_network_port(8000 + (i % 1000) as u16)
                    .unwrap();
                config_manager
                    .set_auto_cleanup_days(30 + (i % 30) as u32)
                    .unwrap();

                let config = config_manager.get_config();
                assert_eq!(config.user.name, format!("stress_user_{}", i));
                assert_eq!(config.network.default_port, 8000 + (i % 1000) as u16);

                let validation_issues = config_manager.validate_config().unwrap();
                assert!(validation_issues.is_empty());

                i
            });
            handles.push(handle);
        }

        let mut results = vec![];
        for handle in handles {
            results.push(handle.await.unwrap());
        }

        let elapsed = start_time.elapsed();
        println!("Config operations stress test completed in {:?}", elapsed);
        println!("Operations: {}", num_operations);
        println!("Average: {:?} per operation", elapsed / num_operations);

        assert_eq!(results.len(), num_operations);

        std::fs::remove_dir_all(&temp_base_dir).unwrap();
    }

    #[tokio::test]
    async fn stress_test_concurrent_sg_link_processing() {
        let num_links = 100;
        let num_processors = 10;

        let peer_source = Peer::new("source_user".to_string(), "127.0.0.1:8080".to_string());
        let crypto_source = Arc::new(RwLock::new(CryptoManager::new().unwrap()));
        let event_bus_source = EventBus::new();
        let contact_manager_source =
            ContactManager::new(peer_source, crypto_source, event_bus_source);

        let mut sg_links = vec![];
        for i in 0..num_links {
            let peer = Peer::new(
                format!("link_user_{}", i),
                format!("127.0.0.1:{}", 8000 + i),
            );
            let crypto = Arc::new(RwLock::new(CryptoManager::new().unwrap()));
            let event_bus = EventBus::new();
            let cm = ContactManager::new(peer, crypto, event_bus);
            let link = cm.generate_sg_link().await.unwrap();
            sg_links.push(link);
        }

        let start_time = Instant::now();
        let semaphore = Arc::new(Semaphore::new(num_processors));
        let mut handles = vec![];

        for (i, link) in sg_links.into_iter().enumerate() {
            let sem = semaphore.clone();
            let peer = Peer::new(
                format!("processor_{}", i),
                format!("127.0.0.1:{}", 9000 + i),
            );
            let crypto = Arc::new(RwLock::new(CryptoManager::new().unwrap()));
            let event_bus = EventBus::new();
            let contact_manager = ContactManager::new(peer, crypto, event_bus);

            let handle = tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();

                match contact_manager.add_contact_by_sg_link(&link).await {
                    Ok(contact) => (i, true, contact.name),
                    Err(_) => (i, false, String::new()),
                }
            });
            handles.push(handle);
        }

        let mut successful = 0;
        for handle in handles {
            let (idx, success, name) = handle.await.unwrap();
            if success {
                successful += 1;
                assert!(name.starts_with("link_user_"));
            }
        }

        let elapsed = start_time.elapsed();
        println!("SG link processing stress test completed in {:?}", elapsed);
        println!("Links processed: {}", num_links);
        println!("Successful: {}", successful);
        println!("Average: {:?} per link", elapsed / num_links);

        assert!(successful >= num_links * 8 / 10);
    }

    #[tokio::test]
    async fn memory_usage_test() {
        let initial_memory = get_memory_usage();
        println!("Initial memory usage: {} MB", initial_memory);

        let num_operations = 500;
        let mut objects = vec![];

        for i in 0..num_operations {
            let test_id = format!("memory_test_{}", i);
            let mut core = ShadowGhostCore::new_for_test(&test_id).unwrap();
            core.initialize(Some(format!("memory_user_{}", i)))
                .await
                .unwrap();

            let sg_link = core.generate_sg_link().await.unwrap();

            let crypto = CryptoManager::new().unwrap();
            let large_data = vec![i as u8; 1024];
            let encrypted = crypto.encrypt(&large_data).unwrap();

            objects.push((core, sg_link, encrypted));

            if i % 100 == 0 {
                let current_memory = get_memory_usage();
                println!("Memory after {} operations: {} MB", i, current_memory);
            }
        }

        let peak_memory = get_memory_usage();
        println!("Peak memory usage: {} MB", peak_memory);

        for (mut core, _, _) in objects {
            core.shutdown().await.unwrap();
        }

        tokio::time::sleep(Duration::from_millis(100)).await;

        let final_memory = get_memory_usage();
        println!("Final memory usage: {} MB", final_memory);

        let memory_increase = peak_memory - initial_memory;
        println!("Memory increase: {} MB", memory_increase);

        assert!(memory_increase < 500);
    }

    fn get_memory_usage() -> u64 {
        #[cfg(target_os = "linux")]
        {
            use std::fs;
            if let Ok(status) = fs::read_to_string("/proc/self/status") {
                for line in status.lines() {
                    if line.starts_with("VmRSS:") {
                        if let Some(kb_str) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = kb_str.parse::<u64>() {
                                return kb / 1024;
                            }
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            if let Ok(output) = Command::new("ps")
                .args(&["-o", "rss=", "-p"])
                .arg(std::process::id().to_string())
                .output()
            {
                if let Ok(rss_str) = String::from_utf8(output.stdout) {
                    if let Ok(kb) = rss_str.trim().parse::<u64>() {
                        return kb / 1024;
                    }
                }
            }
        }

        0
    }
}
