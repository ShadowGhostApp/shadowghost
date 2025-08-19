#[cfg(test)]
mod performance_comparison_tests {
    use shadowghost::prelude::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    use tokio::sync::RwLock;

    mod common;
    use common::{init_test_logging, TestSetup};

    #[tokio::test]
    async fn compare_contact_managers_search_performance() {
        let contact_counts = [100, 1000, 5000];

        for &count in &contact_counts {
            println!("\n=== Testing with {} contacts ===", count);

            let mut contacts = HashMap::new();
            for i in 0..count {
                let contact = Contact {
                    id: format!("perf_contact_id_{:06}", i),
                    name: format!("PerfUser_{:06}", i),
                    address: format!("10.0.{}.{}:{}", (i / 256) % 256, i % 256, 8000 + (i % 1000)),
                    status: if i % 3 == 0 {
                        ContactStatus::Online
                    } else {
                        ContactStatus::Offline
                    },
                    trust_level: match i % 4 {
                        0 => TrustLevel::High,
                        1 => TrustLevel::Medium,
                        2 => TrustLevel::Low,
                        _ => TrustLevel::Unknown,
                    },
                    last_seen: 1234567890 + (i as u64 * 60),
                };
                contacts.insert(format!("perf_contact_id_{:06}", i), contact);
            }

            let peer = Peer::new("perf_test_user".to_string(), "127.0.0.1:8080".to_string());
            let crypto = Arc::new(RwLock::new(CryptoManager::new().unwrap()));
            let event_bus = EventBus::new();

            let original_manager =
                ContactManager::new(peer.clone(), crypto.clone(), event_bus.clone());
            original_manager
                .load_contacts(contacts.clone())
                .await
                .unwrap();

            println!("\n--- Search by Name Performance ---");

            let start = Instant::now();
            for i in 0..100 {
                let target_name = format!("PerfUser_{:06}", i * (count / 100));
                let _ = original_manager.get_contact_by_name(&target_name).await;
            }
            let original_time = start.elapsed();
            println!("Original Manager: {:?} (100 searches)", original_time);

            println!("\n--- Get All Contacts Performance ---");

            let start = Instant::now();
            for _ in 0..10 {
                let _ = original_manager.get_contacts().await;
            }
            let original_get_all_time = start.elapsed();
            println!(
                "Original Manager (get all): {:?} (10 iterations)",
                original_get_all_time
            );

            println!("\n--- Search by ID Performance ---");

            let start = Instant::now();
            for i in 0..100 {
                let target_id = format!("perf_contact_id_{:06}", i * (count / 100));
                let _ = original_manager.get_contact_by_id(&target_id).await;
            }
            let original_id_time = start.elapsed();
            println!(
                "Original Manager (by ID): {:?} (100 searches)",
                original_id_time
            );
        }
    }

    #[tokio::test]
    async fn benchmark_concurrent_search_operations() {
        let count = 5000;
        let mut contacts = HashMap::new();

        for i in 0..count {
            let contact = Contact {
                id: format!("concurrent_perf_id_{:06}", i),
                name: format!("ConcurrentUser_{:06}", i),
                address: format!(
                    "172.16.{}.{}:{}",
                    (i / 256) % 256,
                    i % 256,
                    8000 + (i % 1000)
                ),
                status: if i % 2 == 0 {
                    ContactStatus::Online
                } else {
                    ContactStatus::Offline
                },
                trust_level: TrustLevel::Medium,
                last_seen: 1234567890 + (i as u64 * 30),
            };
            contacts.insert(format!("concurrent_perf_id_{:06}", i), contact);
        }

        let peer = Peer::new(
            "concurrent_perf_user".to_string(),
            "127.0.0.1:8080".to_string(),
        );
        let crypto = Arc::new(RwLock::new(CryptoManager::new().unwrap()));
        let event_bus = EventBus::new();

        let manager = Arc::new(ContactManager::new(peer, crypto, event_bus));
        manager.load_contacts(contacts).await.unwrap();

        println!("\n=== Concurrent Search Performance Test ===");

        let start = Instant::now();
        let mut handles = vec![];

        for i in 0..50 {
            let mgr = manager.clone();
            let handle = tokio::spawn(async move {
                let mut results = vec![];
                for j in 0..20 {
                    let target_name = format!("ConcurrentUser_{:06}", (i * 20 + j) % count);
                    let result = mgr.get_contact_by_name(&target_name).await;
                    results.push(result.is_some());
                }
                results.into_iter().filter(|&x| x).count()
            });
            handles.push(handle);
        }

        let mut total_found = 0;
        for handle in handles {
            total_found += handle.await.unwrap();
        }

        let elapsed = start.elapsed();
        println!(
            "Concurrent searches: {} operations in {:?}",
            50 * 20,
            elapsed
        );
        println!("Found contacts: {}", total_found);
        println!("Average per search: {:?}", elapsed / (50 * 20));
    }

    #[tokio::test]
    async fn memory_usage_comparison() {
        let contact_counts = [1000, 5000, 10000];

        for &count in &contact_counts {
            println!("\n=== Memory Usage Test: {} contacts ===", count);

            let mut contacts = HashMap::new();
            for i in 0..count {
                let contact = Contact {
                    id: format!("memory_test_id_{:06}", i),
                    name: format!("MemoryUser_{:06}", i),
                    address: format!(
                        "192.168.{}.{}:{}",
                        (i / 256) % 256,
                        i % 256,
                        8000 + (i % 1000)
                    ),
                    status: ContactStatus::Online,
                    trust_level: TrustLevel::Medium,
                    last_seen: 1234567890 + (i as u64 * 15),
                };
                contacts.insert(format!("memory_test_id_{:06}", i), contact);
            }

            let initial_memory = get_memory_usage();

            let peer = Peer::new("memory_test_user".to_string(), "127.0.0.1:8080".to_string());
            let crypto = Arc::new(RwLock::new(CryptoManager::new().unwrap()));
            let event_bus = EventBus::new();

            let manager = ContactManager::new(peer, crypto, event_bus);
            manager.load_contacts(contacts).await.unwrap();

            let loaded_memory = get_memory_usage();
            let memory_increase = loaded_memory - initial_memory;

            println!("Memory increase: {} MB", memory_increase);
            println!(
                "Memory per contact: {} KB",
                (memory_increase * 1024) / count as u64
            );

            for _ in 0..1000 {
                let target_name = format!("MemoryUser_{:06}", rand::random::<usize>() % count);
                let _ = manager.get_contact_by_name(&target_name).await;
            }

            let after_search_memory = get_memory_usage();
            println!("Memory after 1000 searches: {} MB", after_search_memory);
        }
    }

    #[tokio::test]
    async fn stress_test_rapid_modifications() {
        let initial_count = 1000;
        let operations = 5000;

        let peer = Peer::new("stress_test_user".to_string(), "127.0.0.1:8080".to_string());
        let crypto = Arc::new(RwLock::new(CryptoManager::new().unwrap()));
        let event_bus = EventBus::new();
        let manager = Arc::new(ContactManager::new(
            peer.clone(),
            crypto.clone(),
            event_bus.clone(),
        ));

        let mut initial_contacts = HashMap::new();
        for i in 0..initial_count {
            let contact = Contact {
                id: format!("stress_id_{:06}", i),
                name: format!("StressUser_{:06}", i),
                address: format!(
                    "10.10.{}.{}:{}",
                    (i / 256) % 256,
                    i % 256,
                    8000 + (i % 1000)
                ),
                status: ContactStatus::Online,
                trust_level: TrustLevel::Medium,
                last_seen: 1234567890,
            };
            initial_contacts.insert(format!("stress_id_{:06}", i), contact);
        }

        manager.load_contacts(initial_contacts).await.unwrap();

        println!("\n=== Stress Test: Rapid Modifications ===");

        let start = Instant::now();
        let mut operation_times = vec![];

        for i in 0..operations {
            let op_start = Instant::now();

            match i % 4 {
                0 => {
                    let target_name = format!("StressUser_{:06}", i % initial_count);
                    let _ = manager.get_contact_by_name(&target_name).await;
                }
                1 => {
                    let target_id = format!("stress_id_{:06}", i % initial_count);
                    let _ = manager.get_contact_by_id(&target_id).await;
                }
                2 => {
                    let contacts = manager.get_contacts().await;
                    assert!(!contacts.is_empty());
                }
                3 => {
                    let online_contacts = manager
                        .get_contacts()
                        .await
                        .into_iter()
                        .filter(|c| c.status == ContactStatus::Online)
                        .collect::<Vec<_>>();
                    let _ = online_contacts.len();
                }
                _ => unreachable!(),
            }

            let op_time = op_start.elapsed();
            operation_times.push(op_time);

            if i % 1000 == 0 && i > 0 {
                let avg_time =
                    operation_times.iter().sum::<Duration>() / operation_times.len() as u32;
                println!("Completed {} operations, avg time: {:?}", i, avg_time);
                operation_times.clear();
            }
        }

        let total_time = start.elapsed();
        println!("Total time for {} operations: {:?}", operations, total_time);
        println!("Average time per operation: {:?}", total_time / operations);
    }

    #[tokio::test]
    async fn benchmark_search_patterns() {
        let count = 10000;
        let mut contacts = HashMap::new();

        for i in 0..count {
            let contact = Contact {
                id: format!("pattern_id_{:06}", i),
                name: format!("PatternUser_{:06}", i),
                address: format!("203.0.113.{}:{}", (i % 254) + 1, 8000 + (i % 1000)),
                status: if i % 5 == 0 {
                    ContactStatus::Online
                } else {
                    ContactStatus::Offline
                },
                trust_level: TrustLevel::Medium,
                last_seen: 1234567890 + (i as u64 * 45),
            };
            contacts.insert(format!("pattern_id_{:06}", i), contact);
        }

        let peer = Peer::new(
            "pattern_test_user".to_string(),
            "127.0.0.1:8080".to_string(),
        );
        let crypto = Arc::new(RwLock::new(CryptoManager::new().unwrap()));
        let event_bus = EventBus::new();
        let manager = ContactManager::new(peer, crypto, event_bus);

        manager.load_contacts(contacts).await.unwrap();

        println!("\n=== Search Pattern Performance ===");

        let patterns = vec![
            ("Sequential search", 0..100),
            (
                "Random search",
                (0..100)
                    .map(|_| rand::random::<usize>() % count)
                    .collect::<Vec<_>>()
                    .into_iter(),
            ),
            ("Clustered search", (5000..5100)),
        ];

        for (pattern_name, indices) in patterns {
            let start = Instant::now();
            let mut found_count = 0;

            for i in indices {
                let target_name = format!("PatternUser_{:06}", i);
                if manager.get_contact_by_name(&target_name).await.is_some() {
                    found_count += 1;
                }
            }

            let elapsed = start.elapsed();
            println!("{}: {:?} (found: {})", pattern_name, elapsed, found_count);
        }

        println!("\n--- Filter Operations ---");

        let start = Instant::now();
        let all_contacts = manager.get_contacts().await;
        let online_contacts: Vec<_> = all_contacts
            .iter()
            .filter(|c| c.status == ContactStatus::Online)
            .collect();
        let filter_time = start.elapsed();

        println!(
            "Filter online contacts: {:?} (found: {})",
            filter_time,
            online_contacts.len()
        );

        let start = Instant::now();
        let pattern_matches: Vec<_> = all_contacts
            .iter()
            .filter(|c| c.name.contains("000"))
            .collect();
        let pattern_filter_time = start.elapsed();

        println!(
            "Filter by name pattern: {:?} (found: {})",
            pattern_filter_time,
            pattern_matches.len()
        );
    }

    #[tokio::test]
    async fn measure_cache_effectiveness() {
        let count = 5000;
        let repeated_searches = 100;

        let mut contacts = HashMap::new();
        for i in 0..count {
            let contact = Contact {
                id: format!("cache_test_id_{:06}", i),
                name: format!("CacheUser_{:06}", i),
                address: format!("198.51.100.{}:{}", (i % 254) + 1, 8000 + (i % 1000)),
                status: ContactStatus::Online,
                trust_level: TrustLevel::Medium,
                last_seen: 1234567890 + (i as u64 * 20),
            };
            contacts.insert(format!("cache_test_id_{:06}", i), contact);
        }

        let peer = Peer::new("cache_test_user".to_string(), "127.0.0.1:8080".to_string());
        let crypto = Arc::new(RwLock::new(CryptoManager::new().unwrap()));
        let event_bus = EventBus::new();
        let manager = ContactManager::new(peer, crypto, event_bus);

        manager.load_contacts(contacts).await.unwrap();

        println!("\n=== Cache Effectiveness Test ===");

        let popular_names: Vec<String> = (0..10)
            .map(|i| format!("CacheUser_{:06}", i * 100))
            .collect();

        println!("First pass (cold cache):");
        let start = Instant::now();
        for _ in 0..repeated_searches {
            for name in &popular_names {
                let _ = manager.get_contact_by_name(name).await;
            }
        }
        let cold_time = start.elapsed();
        println!("Time: {:?}", cold_time);

        println!("Second pass (warm cache):");
        let start = Instant::now();
        for _ in 0..repeated_searches {
            for name in &popular_names {
                let _ = manager.get_contact_by_name(name).await;
            }
        }
        let warm_time = start.elapsed();
        println!("Time: {:?}", warm_time);

        let speedup_ratio = cold_time.as_nanos() as f64 / warm_time.as_nanos() as f64;
        println!("Cache speedup: {:.2}x", speedup_ratio);

        if speedup_ratio > 1.1 {
            println!("✅ Cache is providing performance benefit");
        } else {
            println!("⚠️  Cache benefit not significant");
        }
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
