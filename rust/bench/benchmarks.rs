use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use shadowghost::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::RwLock;

fn bench_crypto_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("crypto_manager_creation", |b| {
        b.iter(|| black_box(CryptoManager::new().unwrap()))
    });

    let crypto = CryptoManager::new().unwrap();

    let mut group = c.benchmark_group("crypto_encrypt_decrypt");

    for size in [16, 64, 256, 1024, 4096].iter() {
        let data = vec![0u8; *size];

        group.bench_with_input(BenchmarkId::new("encrypt", size), size, |b, _| {
            b.iter(|| black_box(crypto.encrypt(black_box(&data)).unwrap()))
        });

        let encrypted = crypto.encrypt(&data).unwrap();
        group.bench_with_input(BenchmarkId::new("decrypt", size), size, |b, _| {
            b.iter(|| black_box(crypto.decrypt(black_box(&encrypted)).unwrap()))
        });
    }

    group.finish();

    let test_data = b"benchmark test message for signing";
    let public_key = crypto.get_public_key();

    c.bench_function("crypto_sign_data", |b| {
        b.iter(|| black_box(crypto.sign_data(black_box(test_data)).unwrap()))
    });

    let signature = crypto.sign_data(test_data).unwrap();
    c.bench_function("crypto_verify_signature", |b| {
        b.iter(|| {
            black_box(
                crypto
                    .verify_signature(
                        black_box(test_data),
                        black_box(&signature),
                        black_box(&public_key),
                    )
                    .unwrap(),
            )
        })
    });

    let crypto2 = CryptoManager::new().unwrap();
    let pub_key2 = crypto2.get_public_key();

    c.bench_function("crypto_derive_shared_secret", |b| {
        b.iter(|| black_box(crypto.derive_shared_secret(black_box(&pub_key2)).unwrap()))
    });
}

fn bench_protocol_operations(c: &mut Criterion) {
    use shadowghost::protocol::*;

    c.bench_function("protocol_create_handshake", |b| {
        b.iter(|| {
            black_box(ProtocolMessage::create_handshake(
                black_box("peer_id".to_string()),
                black_box("peer_name".to_string()),
                black_box("127.0.0.1:8080".to_string()),
                black_box(vec![1, 2, 3, 4]),
            ))
        })
    });

    c.bench_function("protocol_create_text_message", |b| {
        b.iter(|| {
            black_box(ProtocolMessage::create_text_message(
                black_box("sender".to_string()),
                black_box("recipient".to_string()),
                black_box("Hello World".to_string()),
                black_box("msg123".to_string()),
            ))
        })
    });

    let handshake = ProtocolMessage::create_handshake(
        "peer_id".to_string(),
        "peer_name".to_string(),
        "127.0.0.1:8080".to_string(),
        vec![1, 2, 3, 4],
    );

    c.bench_function("protocol_serialize", |b| {
        b.iter(|| black_box(handshake.to_bytes().unwrap()))
    });

    let serialized = handshake.to_bytes().unwrap();
    c.bench_function("protocol_deserialize", |b| {
        b.iter(|| black_box(ProtocolMessage::from_bytes(black_box(&serialized)).unwrap()))
    });

    let mut group = c.benchmark_group("protocol_message_sizes");

    for size in [10, 100, 1000, 10000].iter() {
        let content = "A".repeat(*size);
        let msg = ProtocolMessage::create_text_message(
            "sender".to_string(),
            "recipient".to_string(),
            content,
            "msg_id".to_string(),
        );

        group.bench_with_input(BenchmarkId::new("serialize", size), size, |b, _| {
            b.iter(|| black_box(msg.to_bytes().unwrap()))
        });

        let serialized = msg.to_bytes().unwrap();
        group.bench_with_input(BenchmarkId::new("deserialize", size), size, |b, _| {
            b.iter(|| black_box(ProtocolMessage::from_bytes(black_box(&serialized)).unwrap()))
        });
    }

    group.finish();
}

fn bench_contact_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("contact_manager_creation", |b| {
        b.iter(|| {
            let peer = Peer::new("test_user".to_string(), "127.0.0.1:8080".to_string());
            let crypto = Arc::new(RwLock::new(CryptoManager::new().unwrap()));
            let event_bus = EventBus::new();
            black_box(ContactManager::new(peer, crypto, event_bus))
        })
    });

    let peer = Peer::new("test_user".to_string(), "127.0.0.1:8080".to_string());
    let crypto = Arc::new(RwLock::new(CryptoManager::new().unwrap()));
    let event_bus = EventBus::new();
    let contact_manager = ContactManager::new(peer, crypto, event_bus);

    c.bench_function("generate_sg_link", |b| {
        b.to_async(&rt)
            .iter(|| async { black_box(contact_manager.generate_sg_link().await.unwrap()) })
    });

    let sg_link = rt.block_on(contact_manager.generate_sg_link()).unwrap();

    let peer2 = Peer::new("test_user2".to_string(), "127.0.0.1:8081".to_string());
    let crypto2 = Arc::new(RwLock::new(CryptoManager::new().unwrap()));
    let event_bus2 = EventBus::new();
    let contact_manager2 = ContactManager::new(peer2, crypto2, event_bus2);

    c.bench_function("parse_sg_link", |b| {
        b.to_async(&rt).iter(|| async {
            let _ = contact_manager2
                .add_contact_by_sg_link(black_box(&sg_link))
                .await;
        })
    });
}

fn bench_peer_operations(c: &mut Criterion) {
    c.bench_function("peer_new", |b| {
        b.iter(|| {
            black_box(Peer::new(
                black_box("test_user".to_string()),
                black_box("127.0.0.1:8080".to_string()),
            ))
        })
    });

    c.bench_function("peer_new_with_entropy", |b| {
        b.iter(|| {
            black_box(Peer::new_with_entropy(
                black_box("test_user".to_string()),
                black_box("127.0.0.1:8080".to_string()),
            ))
        })
    });

    let peer = Peer::new("test_user".to_string(), "127.0.0.1:8080".to_string());

    c.bench_function("peer_get_short_id", |b| {
        b.iter(|| black_box(peer.get_short_id()))
    });

    c.bench_function("peer_get_info", |b| b.iter(|| black_box(peer.get_info())));
}

fn bench_storage_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let temp_dir = std::env::temp_dir().join("shadowghost_bench");
    let mut config = AppConfig::default();
    config.storage.data_dir = temp_dir.clone();

    let event_bus = EventBus::new();
    let storage_manager = StorageManager::new(config, event_bus).unwrap();

    let mut contacts = HashMap::new();
    for i in 0..100 {
        let contact = Contact {
            id: format!("id_{}", i),
            name: format!("user_{}", i),
            address: format!("127.0.0.1:808{}", i % 10),
            status: ContactStatus::Online,
            trust_level: TrustLevel::Medium,
            last_seen: 1234567890 + i,
        };
        contacts.insert(format!("id_{}", i), contact);
    }

    c.bench_function("storage_save_contacts", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(
                storage_manager
                    .save_contacts(black_box(&contacts))
                    .await
                    .unwrap(),
            )
        })
    });

    rt.block_on(storage_manager.save_contacts(&contacts))
        .unwrap();

    c.bench_function("storage_load_contacts", |b| {
        b.to_async(&rt)
            .iter(|| async { black_box(storage_manager.load_contacts().await.unwrap()) })
    });

    let messages: Vec<ChatMessage> = (0..100)
        .map(|i| ChatMessage {
            id: format!("msg_{}", i),
            from: "alice".to_string(),
            to: "bob".to_string(),
            content: format!("Message {}", i),
            msg_type: ChatMessageType::Text,
            timestamp: 1234567890 + i,
            delivery_status: DeliveryStatus::Delivered,
        })
        .collect();

    c.bench_function("storage_save_chat_history", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(
                storage_manager
                    .save_chat_history("test_chat", black_box(&messages))
                    .await
                    .unwrap(),
            )
        })
    });

    rt.block_on(storage_manager.save_chat_history("test_chat", &messages))
        .unwrap();

    c.bench_function("storage_load_chat_history", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(
                storage_manager
                    .load_chat_history("test_chat")
                    .await
                    .unwrap(),
            )
        })
    });

    let mut group = c.benchmark_group("storage_message_count");

    for count in [10, 50, 100, 500, 1000].iter() {
        let test_messages: Vec<ChatMessage> = (0..*count)
            .map(|i| ChatMessage {
                id: format!("msg_{}", i),
                from: "alice".to_string(),
                to: "bob".to_string(),
                content: format!("Message {}", i),
                msg_type: ChatMessageType::Text,
                timestamp: 1234567890 + i as u64,
                delivery_status: DeliveryStatus::Delivered,
            })
            .collect();

        group.bench_with_input(BenchmarkId::new("save", count), count, |b, _| {
            b.to_async(&rt).iter(|| async {
                let chat_id = format!("bench_chat_{}", count);
                black_box(
                    storage_manager
                        .save_chat_history(&chat_id, black_box(&test_messages))
                        .await
                        .unwrap(),
                )
            })
        });

        let chat_id = format!("bench_chat_{}", count);
        rt.block_on(storage_manager.save_chat_history(&chat_id, &test_messages))
            .unwrap();

        group.bench_with_input(BenchmarkId::new("load", count), count, |b, _| {
            b.to_async(&rt).iter(|| async {
                black_box(
                    storage_manager
                        .load_chat_history(black_box(&chat_id))
                        .await
                        .unwrap(),
                )
            })
        });
    }

    group.finish();

    let _ = std::fs::remove_dir_all(&temp_dir);
}

fn bench_network_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("network_manager_creation", |b| {
        b.iter(|| {
            let peer = Peer::new("test_user".to_string(), "127.0.0.1:8080".to_string());
            let event_bus = EventBus::new();
            black_box(NetworkManager::new(peer, event_bus).unwrap())
        })
    });

    let peer = Peer::new("test_user".to_string(), "127.0.0.1:8080".to_string());
    let event_bus = EventBus::new();
    let network_manager = NetworkManager::new(peer, event_bus).unwrap();

    c.bench_function("network_get_peer", |b| {
        b.to_async(&rt)
            .iter(|| async { black_box(network_manager.get_peer().await) })
    });

    c.bench_function("network_get_stats", |b| {
        b.to_async(&rt)
            .iter(|| async { black_box(network_manager.get_network_stats().await) })
    });

    c.bench_function("network_get_chats", |b| {
        b.to_async(&rt)
            .iter(|| async { black_box(network_manager.get_chats().await) })
    });
}

fn bench_config_operations(c: &mut Criterion) {
    let temp_dir = std::env::temp_dir().join("shadowghost_config_bench");
    let config_path = temp_dir.join("bench_config.toml");

    std::fs::create_dir_all(&temp_dir).unwrap();

    c.bench_function("config_manager_creation", |b| {
        b.iter(|| {
            let path = temp_dir.join(format!("config_{}.toml", rand::random::<u32>()));
            black_box(ConfigManager::new(black_box(&path)).unwrap())
        })
    });

    let mut config_manager = ConfigManager::new(&config_path).unwrap();

    c.bench_function("config_get_config", |b| {
        b.iter(|| black_box(config_manager.get_config()))
    });

    c.bench_function("config_set_user_name", |b| {
        b.iter(|| {
            black_box(
                config_manager
                    .set_user_name(black_box("bench_user".to_string()))
                    .unwrap(),
            )
        })
    });

    c.bench_function("config_validate", |b| {
        b.iter(|| black_box(config_manager.validate_config().unwrap()))
    });

    let _ = std::fs::remove_dir_all(&temp_dir);
}

fn bench_event_bus_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("event_bus_creation", |b| {
        b.iter(|| black_box(EventBus::new()))
    });

    let event_bus = EventBus::new();
    let test_event = AppEvent::Network(NetworkEvent::ServerStarted { port: 8080 });

    c.bench_function("event_bus_emit", |b| {
        b.iter(|| event_bus.emit(black_box(test_event.clone())))
    });

    c.bench_function("event_bus_subscribe", |b| {
        b.iter(|| black_box(event_bus.subscribe()))
    });

    let mut receiver = event_bus.subscribe();

    c.bench_function("event_bus_emit_and_receive", |b| {
        b.to_async(&rt).iter(|| async {
            event_bus.emit(test_event.clone());
            black_box(receiver.recv().await.unwrap())
        })
    });
}

fn bench_core_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("core_creation", |b| {
        b.iter(|| {
            let test_id = uuid::Uuid::new_v4().to_string();
            black_box(ShadowGhostCore::new_for_test(black_box(&test_id)).unwrap())
        })
    });

    let test_id = uuid::Uuid::new_v4().to_string();
    let mut core = ShadowGhostCore::new_for_test(&test_id).unwrap();
    rt.block_on(core.initialize(Some("bench_user".to_string())))
        .unwrap();

    c.bench_function("core_get_peer_info", |b| {
        b.to_async(&rt)
            .iter(|| async { black_box(core.get_peer_info().await) })
    });

    c.bench_function("core_get_server_status", |b| {
        b.to_async(&rt)
            .iter(|| async { black_box(core.get_server_status().await) })
    });

    c.bench_function("core_generate_sg_link", |b| {
        b.to_async(&rt)
            .iter(|| async { black_box(core.generate_sg_link().await.unwrap()) })
    });

    c.bench_function("core_get_contacts", |b| {
        b.to_async(&rt)
            .iter(|| async { black_box(core.get_contacts().await.unwrap()) })
    });
}

fn bench_tls_masking_operations(c: &mut Criterion) {
    use shadowghost::tls_masking::TlsMaskedConnection;

    let domain = "example.com";

    c.bench_function("tls_create_fake_client_hello", |b| {
        b.iter(|| {
            let fake_stream = std::net::TcpStream::connect("127.0.0.1:1").unwrap_err();
            black_box(fake_stream)
        })
    });

    c.bench_function("tls_create_sni_extension", |b| {
        b.iter(|| {
            let data = format!("fake_sni_extension_for_{}", black_box(domain));
            black_box(data)
        })
    });
}

criterion_group!(
    benches,
    bench_crypto_operations,
    bench_protocol_operations,
    bench_contact_operations,
    bench_peer_operations,
    bench_storage_operations,
    bench_network_operations,
    bench_config_operations,
    bench_event_bus_operations,
    bench_core_operations,
    bench_tls_masking_operations
);

criterion_main!(benches);
