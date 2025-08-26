use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use shadowghost::network::ProtocolMessage;
use shadowghost::prelude::*;
use shadowghost::security::CryptoManager;

fn crypto_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("crypto");

    let crypto = CryptoManager::new().unwrap();
    let test_data = b"Hello, this is test data for encryption benchmarks!";

    group.bench_function("encrypt_decrypt", |b| {
        b.iter(|| {
            let encrypted = crypto.encrypt(test_data).unwrap();
            let _decrypted = crypto.decrypt(&encrypted).unwrap();
        })
    });

    group.bench_function("sign_verify", |b| {
        let public_key = crypto.get_public_key();
        b.iter(|| {
            let signature = crypto.sign_data(test_data).unwrap();
            let _is_valid = crypto
                .verify_signature(test_data, &signature, &public_key)
                .unwrap();
        })
    });

    // Benchmark different data sizes
    for size in [1024, 4096, 16384, 65536].iter() {
        let data = vec![0u8; *size];
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::new("encrypt", size), size, |b, _| {
            b.iter(|| crypto.encrypt(&data).unwrap())
        });
    }

    group.finish();
}

fn protocol_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("protocol");

    group.bench_function("create_text_message", |b| {
        b.iter(|| {
            let _msg = ProtocolMessage::create_text_message(
                "sender".to_string(),
                "recipient".to_string(),
                "Hello, world!".to_string(),
                uuid::Uuid::new_v4().to_string(),
            );
        })
    });

    group.bench_function("serialize_deserialize", |b| {
        let msg = ProtocolMessage::create_text_message(
            "sender".to_string(),
            "recipient".to_string(),
            "Hello, world!".to_string(),
            uuid::Uuid::new_v4().to_string(),
        );

        b.iter(|| {
            let bytes = msg.to_bytes().unwrap();
            let _restored = ProtocolMessage::from_bytes(&bytes).unwrap();
        })
    });

    // Benchmark different message sizes
    for size in [100, 1000, 10000].iter() {
        let content = "x".repeat(*size);
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::new("large_message", size), size, |b, _| {
            b.iter(|| {
                let msg = ProtocolMessage::create_text_message(
                    "sender".to_string(),
                    "recipient".to_string(),
                    content.clone(),
                    uuid::Uuid::new_v4().to_string(),
                );
                let _bytes = msg.to_bytes().unwrap();
            })
        });
    }

    group.finish();
}

fn contact_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("contacts");

    group.bench_function("sg_link_generation", |b| {
        use shadowghost::core::Peer;
        use shadowghost::data::ContactManager;

        let peer = Peer::new("Test User".to_string(), "127.0.0.1:8080".to_string());
        let contact_manager = ContactManager::new(peer);

        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let _link = contact_manager.generate_sg_link().await.unwrap();
            });
        })
    });

    group.bench_function("contact_search", |b| {
        use shadowghost::core::Peer;
        use shadowghost::data::ContactManager;
        use shadowghost::network::{Contact, ContactStatus, TrustLevel};

        let peer = Peer::new("Test User".to_string(), "127.0.0.1:8080".to_string());
        let mut contact_manager = ContactManager::new(peer);

        // Add many contacts for search testing
        for i in 0..1000 {
            let contact = Contact {
                id: format!("contact_{}", i),
                name: format!("Contact {}", i),
                address: format!("127.0.0.1:808{}", i % 10),
                status: ContactStatus::Offline,
                trust_level: TrustLevel::Unknown,
                last_seen: Some(chrono::Utc::now()),
            };
            contact_manager.add_contact(contact).unwrap();
        }

        b.iter(|| {
            let _results = contact_manager.search_contacts("Contact 5");
        })
    });

    group.finish();
}

fn storage_benchmarks(c: &mut Criterion) {
    use shadowghost::core::AppConfig;
    use shadowghost::data::StorageManager;
    use shadowghost::network::{ChatMessage, ChatMessageType, DeliveryStatus};
    use tempfile::TempDir;

    let mut group = c.benchmark_group("storage");

    group.bench_function("save_load_messages", |b| {
        let temp_dir = TempDir::new().unwrap();
        let config = AppConfig {
            storage: shadowghost::core::StorageConfig {
                data_path: temp_dir.path().to_string_lossy().to_string(),
                ..Default::default()
            },
            ..Default::default()
        };

        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let mut storage =
                    StorageManager::new(config.clone(), shadowghost::EventBus::new()).unwrap();
                storage.initialize().await.unwrap();

                let message = ChatMessage {
                    id: uuid::Uuid::new_v4().to_string(),
                    from: "test_sender".to_string(),
                    to: "test_recipient".to_string(),
                    content: "Benchmark test message".to_string(),
                    msg_type: ChatMessageType::Text,
                    timestamp: chrono::Utc::now().timestamp() as u64,
                    delivery_status: DeliveryStatus::Sent,
                };

                storage.save_message("test_chat", &message).await.unwrap();
                let _messages = storage.get_messages("test_chat").await.unwrap();
            });
        })
    });

    group.finish();
}

fn network_benchmarks(c: &mut Criterion) {
    use shadowghost::core::Peer;
    use shadowghost::network::NetworkManager;
    use shadowghost::EventBus;

    let mut group = c.benchmark_group("network");

    group.bench_function("peer_connection_setup", |b| {
        b.iter(|| {
            let peer = Peer::new("Benchmark User".to_string(), "127.0.0.1:8080".to_string());
            let event_bus = EventBus::new();
            let _network = NetworkManager::new(peer, event_bus).unwrap();
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    crypto_benchmarks,
    protocol_benchmarks,
    contact_benchmarks,
    storage_benchmarks,
    network_benchmarks
);
criterion_main!(benches);
