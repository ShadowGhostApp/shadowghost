mod common;
#[path = "common/events.rs"]
mod events;
use common::*;
use events::*;
use std::time::Duration;

#[tokio::test]
async fn test_message_sending_and_receiving() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();

    let setup1 = TestSetup::new("alice").await?;
    let setup2 = TestSetup::new("bob").await?;

    println!("🧪 Testing message exchange between isolated instances");

    let sg_link1 = setup1.core.generate_sg_link().await?;
    let sg_link2 = setup2.core.generate_sg_link().await?;

    println!("🔗 Generated SG links for both users");

    setup2.core.add_contact_by_sg_link(&sg_link1).await?;
    setup1.core.add_contact_by_sg_link(&sg_link2).await?;

    tokio::time::sleep(Duration::from_millis(1000)).await;

    let contacts1 = setup1.core.get_contacts().await?;
    let contacts2 = setup2.core.get_contacts().await?;
    assert_eq!(contacts1.len(), 1, "Alice should have 1 contact");
    assert_eq!(contacts2.len(), 1, "Bob should have 1 contact");
    assert_eq!(contacts1[0].name, "bob", "Alice's contact should be Bob");
    assert_eq!(contacts2[0].name, "alice", "Bob's contact should be Alice");

    println!("✅ Contacts added successfully");

    let receiver1 = setup1.get_event_receiver();

    let test_message = "Hello from Bob to Alice";
    println!("📤 Sending message: '{}'", test_message);
    setup2.core.send_message("alice", test_message).await?;

    let received =
        wait_for_message_received(receiver1, test_message, Duration::from_secs(10)).await?;
    assert_eq!(
        received.content, test_message,
        "Received message should match sent message"
    );

    println!("📨 Message received successfully");

    let chat1 = setup1.core.get_chat_messages("bob").await?;
    let chat2 = setup2.core.get_chat_messages("alice").await?;
    assert!(
        !chat1.is_empty(),
        "Alice's chat history should not be empty"
    );
    assert!(!chat2.is_empty(), "Bob's chat history should not be empty");
    assert_eq!(
        chat1.last().unwrap().content,
        test_message,
        "Last message in Alice's chat should match"
    );
    assert_eq!(
        chat2.last().unwrap().content,
        test_message,
        "Last message in Bob's chat should match"
    );

    println!("💾 Chat history verified");

    setup1.shutdown().await?;
    setup2.shutdown().await?;

    println!("🧹 Test cleanup completed");

    Ok(())
}

#[tokio::test]
async fn test_message_to_unknown_contact() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();

    let setup1 = TestSetup::new("user1").await?;

    println!("🧪 Testing message to unknown contact");

    let result = setup1
        .core
        .send_message("unknown_user", "Hello unknown")
        .await;
    assert!(result.is_err(), "Sending to unknown contact should fail");

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("not found") || error_msg.contains("Contact"),
        "Error message should indicate contact not found: {}",
        error_msg
    );

    println!("✅ Unknown contact error handled correctly: {}", error_msg);

    setup1.shutdown().await?;

    Ok(())
}

#[tokio::test]
async fn test_contact_online_status_check() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();

    let setup1 = TestSetup::new("checker").await?;
    let setup2 = TestSetup::new("target").await?;

    println!("🧪 Testing contact online status detection");

    let sg_link2 = setup2.core.generate_sg_link().await?;
    setup1.core.add_contact_by_sg_link(&sg_link2).await?;

    tokio::time::sleep(Duration::from_millis(500)).await;

    let is_online_when_running = setup1.core.check_contact_online("target").await;
    println!(
        "📡 Contact status when server running: {}",
        is_online_when_running
    );

    setup2.shutdown().await?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    let is_online_after_shutdown = setup1.core.check_contact_online("target").await;
    println!(
        "📡 Contact status after shutdown: {}",
        is_online_after_shutdown
    );

    assert!(
        !is_online_after_shutdown,
        "Contact should be offline after shutdown"
    );

    setup1.shutdown().await?;

    println!("✅ Online status detection working correctly");

    Ok(())
}

#[tokio::test]
async fn test_contact_persistence() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();

    let test_id = "persistence_test";

    {
        let setup1 = TestSetup::new(&format!("{}_user1", test_id)).await?;
        let setup2 = TestSetup::new(&format!("{}_user2", test_id)).await?;

        println!("🧪 Testing contact persistence across sessions");

        let sg_link2 = setup2.core.generate_sg_link().await?;
        setup1.core.add_contact_by_sg_link(&sg_link2).await?;

        let contacts_before = setup1.core.get_contacts().await?;
        assert_eq!(
            contacts_before.len(),
            1,
            "Should have 1 contact before restart"
        );

        println!("💾 Contact saved, shutting down first instance");
        setup1.shutdown().await?;
        setup2.shutdown().await?;
    }

    tokio::time::sleep(Duration::from_millis(100)).await;

    {
        let setup1_new = TestSetup::new(&format!("{}_user1", test_id)).await?;

        println!("🔄 Restarted instance, checking persisted contacts");

        let contacts_after = setup1_new.core.get_contacts().await?;

        println!("📊 Contacts found after restart: {}", contacts_after.len());

        setup1_new.shutdown().await?;
    }

    println!("✅ Contact persistence test completed");

    Ok(())
}
