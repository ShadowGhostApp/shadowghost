// tests/validation_tests.rs
use shadowghost::contacts::{generate_sg_link, parse_sg_link, ContactManager};
use shadowghost::core::{Engine, Peer, Profile};
use shadowghost::events::EventBus;
use shadowghost::network::{Contact, ContactStatus, TrustLevel};
use shadowghost::storage::StorageManager;
use std::time::Duration;

struct TestSetup {
    temp_dir: std::path::PathBuf,
    contact_manager: ContactManager,
    peer: Peer,
}

impl TestSetup {
    async fn new(name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let test_id = format!("{}-{}", name, uuid::Uuid::new_v4());
        let temp_dir = std::env::temp_dir().join("shadowghost_test").join(&test_id);
        std::fs::create_dir_all(&temp_dir)?;

        let contact_manager = ContactManager::new(&temp_dir)?;
        let peer = Peer::new(
            name.to_string(),
            format!("127.0.0.1:{}", 8000 + rand::random::<u16>() % 1000),
        );

        Ok(Self {
            temp_dir,
            contact_manager,
            peer,
        })
    }

    async fn generate_sg_link(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(generate_sg_link(&self.peer)?)
    }

    async fn shutdown(self) -> Result<(), Box<dyn std::error::Error>> {
        if self.temp_dir.exists() {
            std::fs::remove_dir_all(&self.temp_dir)?;
        }
        Ok(())
    }
}

#[tokio::test]
async fn test_invalid_sg_link_handling() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing invalid SG link handling");

    let setup1 = TestSetup::new("validator").await?;

    let invalid_links = vec![
        ("empty", ""),
        ("not-sg-link", "invalid-link"),
        ("sg-no-data", "sg://"),
        ("sg-invalid-base64", "sg://invalid-base64-!@#$%^&*()"),
        ("sg-not-json", "sg://dGhpcyBpcyBub3QganNvbg=="), // "this is not json"
        ("sg-invalid-json", "sg://eyJpbnZhbGlkIjogImpzb24ifQ=="), // {"invalid": "json"}
        ("http-link", "http://not-sg-link.com"),
        ("sg-empty-data", "sg://YWJjZGVmZ2hpams="), // "abcdefghijk"
        ("sg-malformed", "sg://xyz123"),
        ("sg-partial", "sg://eyJ"),
    ];

    println!("🔍 Testing {} invalid link formats", invalid_links.len());

    for (test_name, invalid_link) in invalid_links {
        println!("  Testing {}: '{}'", test_name, invalid_link);

        let result = parse_sg_link(invalid_link, "validator");
        assert!(
            result.is_err(),
            "Invalid link '{}' should be rejected but was accepted",
            invalid_link
        );

        let error_msg = result.unwrap_err().to_string();
        println!("    ✅ Rejected with: {}", error_msg);
    }

    setup1.shutdown().await?;
    println!("✅ All invalid links properly rejected");

    Ok(())
}

#[tokio::test]
async fn test_self_addition_prevention() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing prevention of self-addition");

    let setup1 = TestSetup::new("narcissist").await?;

    let sg_link1 = setup1.generate_sg_link().await?;
    println!(
        "🔗 Generated own SG link: {}",
        &sg_link1[..50.min(sg_link1.len())]
    );

    let result = parse_sg_link(&sg_link1, "narcissist");
    assert!(result.is_err(), "Should not be able to add self as contact");

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("Cannot add yourself") || error_msg.contains("yourself as contact"),
        "Error message should mention self-addition prevention: {}",
        error_msg
    );

    println!("✅ Self-addition properly prevented: {}", error_msg);

    setup1.shutdown().await?;

    Ok(())
}

#[tokio::test]
async fn test_sg_link_generation_consistency() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing SG link generation consistency");

    let setup1 = TestSetup::new("consistent_user").await?;

    let link1 = setup1.generate_sg_link().await?;
    let link2 = setup1.generate_sg_link().await?;
    let link3 = setup1.generate_sg_link().await?;

    println!("🔗 Generated 3 links for consistency check");

    assert!(link1.starts_with("sg://"), "Link 1 should start with sg://");
    assert!(link2.starts_with("sg://"), "Link 2 should start with sg://");
    assert!(link3.starts_with("sg://"), "Link 3 should start with sg://");

    assert_eq!(link1, link2, "Link 1 and 2 should be identical");
    assert_eq!(link2, link3, "Link 2 and 3 should be identical");
    assert_eq!(link1, link3, "Link 1 and 3 should be identical");

    assert!(
        link1.len() > 20,
        "Link should be reasonably long (>20 chars)"
    );
    assert!(
        link2.len() > 20,
        "Link should be reasonably long (>20 chars)"
    );

    println!("✅ Link length: {} characters", link1.len());
    println!("✅ All links are identical and properly formatted");

    let link_data = &link1[5..]; // Remove "sg://" prefix
    let is_valid_base64 = link_data.chars().all(|c| {
        c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=' || c == '-' || c == '_'
    });
    assert!(is_valid_base64, "Link data should be valid base64");

    println!("✅ Link contains valid base64 data");

    setup1.shutdown().await?;

    Ok(())
}

#[tokio::test]
async fn test_duplicate_contact_handling() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing duplicate contact handling");

    let mut setup1 = TestSetup::new("collector").await?;
    let setup2 = TestSetup::new("target").await?;

    let sg_link2 = setup2.generate_sg_link().await?;

    println!("📇 Adding contact for the first time");
    let contact = parse_sg_link(&sg_link2, "collector")?;
    setup1.contact_manager.add_contact(contact.clone())?;

    let contacts_first = setup1.contact_manager.get_contacts();
    assert_eq!(
        contacts_first.len(),
        1,
        "Should have 1 contact after first addition"
    );

    println!("📇 Adding the same contact again");
    let contact2 = parse_sg_link(&sg_link2, "collector")?;
    let result = setup1.contact_manager.add_contact(contact2);
    // This should either succeed (updating existing) or fail gracefully

    let contacts_second = setup1.contact_manager.get_contacts();
    assert_eq!(
        contacts_second.len(),
        1,
        "Should still have exactly 1 contact after duplicate addition"
    );

    assert_eq!(
        contacts_first[0].name, contacts_second[0].name,
        "Contact name should remain the same"
    );
    assert_eq!(
        contacts_first[0].id, contacts_second[0].id,
        "Contact ID should remain the same"
    );

    println!("✅ Duplicate contact handled correctly - no duplicates created");
    println!("  Contact name: {}", contacts_second[0].name);
    println!("  Contact ID: {}", contacts_second[0].id);

    setup1.shutdown().await?;
    setup2.shutdown().await?;

    Ok(())
}

#[tokio::test]
async fn test_message_content_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing message content validation");

    use shadowghost::utils::validation::validate_message_content;

    // Test empty message
    let result = validate_message_content("");
    assert!(result.is_err(), "Empty message should be invalid");

    // Test normal message
    let result = validate_message_content("Hello, world!");
    assert!(result.is_ok(), "Normal message should be valid");

    // Test very long message
    let long_message = "A".repeat(10001);
    let result = validate_message_content(&long_message);
    assert!(result.is_err(), "Very long message should be invalid");

    println!("✅ Message content validation working correctly");

    Ok(())
}

#[tokio::test]
async fn test_malformed_link_decoding() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing malformed link decoding");

    let malformed_links = vec![
        ("corrupted-base64", "sg://abc123!@#"),
        ("incomplete-base64", "sg://eyJ"),
        ("wrong-padding", "sg://eyJhIjoiYiJ"), // Missing padding
        ("valid-base64-invalid-json", "sg://dGhpcyBpcyBub3QgSlNPTg=="), // "this is not JSON"
        ("json-missing-fields", "sg://eyJvbmx5IjoiZmllbGQifQ=="), // {"only":"field"}
    ];

    for (test_name, malformed_link) in malformed_links {
        println!("🔍 Testing {}: {}", test_name, malformed_link);

        let result = parse_sg_link(malformed_link, "decoder");
        assert!(
            result.is_err(),
            "Malformed link '{}' should be rejected",
            malformed_link
        );

        let error = result.unwrap_err().to_string();
        println!("  ✅ Rejected: {}", error);
    }

    println!("✅ All malformed links properly rejected with appropriate errors");

    Ok(())
}

#[tokio::test]
async fn test_contact_validation_edge_cases() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing contact validation edge cases");

    let mut setup1 = TestSetup::new("edge_tester").await?;

    let setup2 = TestSetup::new("rapid1").await?;
    let setup3 = TestSetup::new("rapid2").await?;
    let setup4 = TestSetup::new("rapid3").await?;

    let link2 = setup2.generate_sg_link().await?;
    let link3 = setup3.generate_sg_link().await?;
    let link4 = setup4.generate_sg_link().await?;

    println!("📇 Rapidly adding multiple contacts");

    let contact2 = parse_sg_link(&link2, "edge_tester")?;
    let contact3 = parse_sg_link(&link3, "edge_tester")?;
    let contact4 = parse_sg_link(&link4, "edge_tester")?;

    setup1.contact_manager.add_contact(contact2)?;
    setup1.contact_manager.add_contact(contact3)?;
    setup1.contact_manager.add_contact(contact4)?;

    let contacts = setup1.contact_manager.get_contacts();
    assert_eq!(contacts.len(), 3, "Should have exactly 3 contacts");

    let contact_names: Vec<String> = contacts.iter().map(|c| c.name.clone()).collect();
    println!("✅ Added contacts: {:?}", contact_names);

    assert!(contact_names.contains(&"rapid1".to_string()));
    assert!(contact_names.contains(&"rapid2".to_string()));
    assert!(contact_names.contains(&"rapid3".to_string()));

    setup1.shutdown().await?;
    setup2.shutdown().await?;
    setup3.shutdown().await?;
    setup4.shutdown().await?;

    println!("✅ Edge case validation completed successfully");

    Ok(())
}

#[tokio::test]
async fn test_address_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing address validation");

    use shadowghost::utils::connection::ConnectionUtils;

    // Test valid addresses
    let valid_addresses = vec![
        "127.0.0.1:8080",
        "192.168.1.1:3000",
        "example.com:80",
        "localhost:8080",
    ];

    for address in valid_addresses {
        assert!(
            ConnectionUtils::validate_peer_address(address),
            "Address '{}' should be valid",
            address
        );
    }

    // Test invalid addresses
    let invalid_addresses = vec![
        "",
        "invalid-address",
        "127.0.0.1",
        ":8080",
        "127.0.0.1:",
        "127.0.0.1:abc",
    ];

    for address in invalid_addresses {
        assert!(
            !ConnectionUtils::validate_peer_address(address),
            "Address '{}' should be invalid",
            address
        );
    }

    println!("✅ Address validation working correctly");

    Ok(())
}

#[tokio::test]
async fn test_contact_name_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing contact name validation");

    use shadowghost::utils::validation::validate_contact_name;

    // Test valid names
    let valid_names = vec!["Alice", "Bob123", "User_Name", "Test User"];
    for name in valid_names {
        assert!(
            validate_contact_name(name).is_ok(),
            "Name '{}' should be valid",
            name
        );
    }

    // Test invalid names
    let invalid_names = vec![
        "",                     // Empty
        " ",                    // Whitespace only
        &"A".repeat(51),        // Too long
        "Name\nWith\nNewlines", // Contains newlines
        "Name\tWith\tTabs",     // Contains tabs
    ];

    for name in invalid_names {
        assert!(
            validate_contact_name(name).is_err(),
            "Name '{}' should be invalid",
            name
        );
    }

    println!("✅ Contact name validation working correctly");

    Ok(())
}

#[tokio::test]
async fn test_storage_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing storage validation");

    let temp_dir = std::env::temp_dir().join("shadowghost_storage_validation_test");
    std::fs::create_dir_all(&temp_dir)?;

    let event_bus = EventBus::new();
    let storage_manager = StorageManager::new(&temp_dir, event_bus)?;
    storage_manager.initialize().await?;

    // Validate empty storage
    let contact_issues = storage_manager.validate_contacts().await?;
    let chat_issues = storage_manager.validate_chats().await?;

    assert!(
        contact_issues.is_empty(),
        "Empty storage should have no contact issues"
    );
    assert!(
        chat_issues.is_empty(),
        "Empty storage should have no chat issues"
    );

    // Add some valid data
    let contact = Contact {
        id: "valid_contact".to_string(),
        name: "Valid User".to_string(),
        address: "127.0.0.1:8080".to_string(),
        status: ContactStatus::Online,
        trust_level: TrustLevel::Trusted,
        last_seen: Some(chrono::Utc::now()),
    };

    storage_manager.save_contact(&contact).await?;

    // Validate again
    let contact_issues = storage_manager.validate_contacts().await?;
    assert!(
        contact_issues.is_empty(),
        "Valid contact should have no issues"
    );

    std::fs::remove_dir_all(&temp_dir)?;

    println!("✅ Storage validation working correctly");

    Ok(())
}
