mod common;

use common::*;
use std::time::Duration;

#[tokio::test]
async fn test_invalid_sg_link_handling() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();

    let setup1 = TestSetup::new("validator").await?;

    println!("🧪 Testing invalid SG link handling");

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

        let result = setup1.core.add_contact_by_sg_link(invalid_link).await;
        assert!(
            result.is_err(),
            "Invalid link '{}' should be rejected but was accepted",
            invalid_link
        );

        let error_msg = result.unwrap_err().to_string();
        println!("    ✅ Rejected with: {}", error_msg);
    }

    let contacts = setup1.core.get_contacts().await?;
    assert_eq!(
        contacts.len(),
        0,
        "No contacts should be added from invalid links"
    );

    setup1.shutdown().await?;

    println!("✅ All invalid links properly rejected");

    Ok(())
}

#[tokio::test]
async fn test_self_addition_prevention() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();

    let setup1 = TestSetup::new("narcissist").await?;

    println!("🧪 Testing prevention of self-addition");

    let sg_link1 = setup1.core.generate_sg_link().await?;
    println!("🔗 Generated own SG link: {}", &sg_link1[..50]);

    let result = setup1.core.add_contact_by_sg_link(&sg_link1).await;
    assert!(result.is_err(), "Should not be able to add self as contact");

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("Cannot add yourself") || error_msg.contains("yourself as contact"),
        "Error message should mention self-addition prevention: {}",
        error_msg
    );

    println!("✅ Self-addition properly prevented: {}", error_msg);

    let contacts = setup1.core.get_contacts().await?;
    assert_eq!(
        contacts.len(),
        0,
        "No contacts should be added when trying to add self"
    );

    setup1.shutdown().await?;

    Ok(())
}

#[tokio::test]
async fn test_sg_link_generation_consistency() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();

    let setup1 = TestSetup::new("consistent_user").await?;

    println!("🧪 Testing SG link generation consistency");

    let link1 = setup1.core.generate_sg_link().await?;
    let link2 = setup1.core.generate_sg_link().await?;
    let link3 = setup1.core.generate_sg_link().await?;

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
    init_test_logging();

    let setup1 = TestSetup::new("collector").await?;
    let setup2 = TestSetup::new("target").await?;

    println!("🧪 Testing duplicate contact handling");

    let sg_link2 = setup2.core.generate_sg_link().await?;

    println!("📇 Adding contact for the first time");
    setup1.core.add_contact_by_sg_link(&sg_link2).await?;
    let contacts_first = setup1.core.get_contacts().await?;
    assert_eq!(
        contacts_first.len(),
        1,
        "Should have 1 contact after first addition"
    );

    println!("📇 Adding the same contact again");
    let _result: Result<(), shadowghost::CoreError> =
        setup1.core.add_contact_by_sg_link(&sg_link2).await;

    // Should either succeed (update) or fail gracefully
    let contacts_second = setup1.core.get_contacts().await?;
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
async fn test_empty_and_long_message_handling() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();

    let setup1 = TestSetup::new("receiver").await?;
    let setup2 = TestSetup::new("sender").await?;

    println!("🧪 Testing empty and long message handling");

    let sg_link1 = setup1.core.generate_sg_link().await?;
    setup2.core.add_contact_by_sg_link(&sg_link1).await?;
    tokio::time::sleep(Duration::from_millis(1000)).await;

    println!("📤 Testing empty message");
    let result_empty = setup2.core.send_message("receiver", "").await;
    println!("  Empty message result: {:?}", result_empty.is_ok());

    println!("📤 Testing very long message");
    let long_message = "A".repeat(10000);
    let result_long = setup2.core.send_message("receiver", &long_message).await;
    println!("  Long message result: {:?}", result_long.is_ok());

    tokio::time::sleep(Duration::from_millis(1000)).await;

    let chat_history = setup1.core.get_chat_messages("sender").await?;
    println!("💾 Chat history contains {} messages", chat_history.len());

    for (i, msg) in chat_history.iter().enumerate() {
        let content_preview = if msg.content.len() > 50 {
            format!("{}... ({} chars)", &msg.content[..50], msg.content.len())
        } else {
            msg.content.clone()
        };
        println!("  Message {}: '{}'", i + 1, content_preview);
    }

    println!("✅ Empty and long message handling completed");

    setup1.shutdown().await?;
    setup2.shutdown().await?;

    Ok(())
}

#[tokio::test]
async fn test_malformed_link_decoding() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();

    let setup1 = TestSetup::new("decoder").await?;

    println!("🧪 Testing malformed link decoding");

    let malformed_links = vec![
        ("corrupted-base64", "sg://abc123!@#"),
        ("incomplete-base64", "sg://eyJ"),
        ("wrong-padding", "sg://eyJhIjoiYiJ"), // Missing padding
        ("non-utf8", "sg://w6zDrMOt"),         // Invalid UTF-8 sequence
        ("valid-base64-invalid-json", "sg://dGhpcyBpcyBub3QgSlNPTg=="), // "this is not JSON"
        ("json-missing-fields", "sg://eyJvbmx5IjoiZmllbGQifQ=="), // {"only":"field"}
        ("json-wrong-types", "sg://eyJpZCI6MTIzLCJuYW1lIjp0cnVlfQ=="), // {"id":123,"name":true}
    ];

    for (test_name, malformed_link) in malformed_links {
        println!("🔍 Testing {}: {}", test_name, malformed_link);

        let result = setup1.core.add_contact_by_sg_link(malformed_link).await;
        assert!(
            result.is_err(),
            "Malformed link '{}' should be rejected",
            malformed_link
        );

        let error = result.unwrap_err().to_string();
        println!("  ✅ Rejected: {}", error);
    }

    println!("✅ All malformed links properly rejected with appropriate errors");

    setup1.shutdown().await?;

    Ok(())
}

#[tokio::test]
async fn test_contact_validation_edge_cases() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();

    let setup1 = TestSetup::new("edge_tester").await?;

    println!("🧪 Testing contact validation edge cases");

    // Test multiple rapid additions of different contacts
    let setup2 = TestSetup::new("rapid1").await?;
    let setup3 = TestSetup::new("rapid2").await?;
    let setup4 = TestSetup::new("rapid3").await?;

    let link2 = setup2.core.generate_sg_link().await?;
    let link3 = setup3.core.generate_sg_link().await?;
    let link4 = setup4.core.generate_sg_link().await?;

    println!("📇 Rapidly adding multiple contacts");

    setup1.core.add_contact_by_sg_link(&link2).await?;
    setup1.core.add_contact_by_sg_link(&link3).await?;
    setup1.core.add_contact_by_sg_link(&link4).await?;

    let contacts = setup1.core.get_contacts().await?;
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
