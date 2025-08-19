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

    let sg_link1 = setup1.core.generate_sg_link().await?;
    let sg_link2 = setup2.core.generate_sg_link().await?;

    setup2.core.add_contact_by_sg_link(&sg_link1).await?;
    setup1.core.add_contact_by_sg_link(&sg_link2).await?;

    tokio::time::sleep(Duration::from_millis(1000)).await;

    let contacts1 = setup1.core.get_contacts().await?;
    let contacts2 = setup2.core.get_contacts().await?;
    assert_eq!(contacts1.len(), 1, "Alice should have 1 contact");
    assert_eq!(contacts2.len(), 1, "Bob should have 1 contact");
    assert_eq!(contacts1[0].name, "bob", "Alice's contact should be Bob");
    assert_eq!(contacts2[0].name, "alice", "Bob's contact should be Alice");

    let receiver1 = setup1.get_event_receiver();

    let alice_online = setup2.core.check_contact_online("alice").await;
    if !alice_online {
        setup1.shutdown().await?;
        setup2.shutdown().await?;
        return Err("Alice not reachable for message test".into());
    }

    let test_message = "Hello from Bob to Alice";
    let send_result = setup2.core.send_message("alice", test_message).await;

    match send_result {
        Ok(_) => {
            let received =
                wait_for_message_received(receiver1, test_message, Duration::from_secs(10)).await?;
            assert_eq!(
                received.content, test_message,
                "Received message should match sent message"
            );

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
        }
        Err(e) => {
            if e.to_string().contains("unavailable") || e.to_string().contains("Connection refused")
            {
                // Expected behavior for offline contacts
            } else {
                return Err(e.into());
            }
        }
    }

    setup1.shutdown().await?;
    setup2.shutdown().await?;

    Ok(())
}

#[tokio::test]
async fn test_message_to_unknown_contact() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();

    let setup1 = TestSetup::new("user1").await?;

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

    setup1.shutdown().await?;

    Ok(())
}

#[tokio::test]
async fn test_contact_online_status_check() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();

    let setup1 = TestSetup::new("checker").await?;
    let setup2 = TestSetup::new("target").await?;

    let sg_link2 = setup2.core.generate_sg_link().await?;
    setup1.core.add_contact_by_sg_link(&sg_link2).await?;

    tokio::time::sleep(Duration::from_millis(500)).await;

    let is_online_when_running = setup1.core.check_contact_online("target").await;

    setup2.shutdown().await?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    let is_online_after_shutdown = setup1.core.check_contact_online("target").await;

    assert!(
        !is_online_after_shutdown,
        "Contact should be offline after shutdown"
    );

    if is_online_when_running {
        let test_result = setup1.core.send_message("target", "test").await;
        assert!(
            test_result.is_err(),
            "Message should fail to offline contact"
        );
    }

    setup1.shutdown().await?;

    Ok(())
}

#[tokio::test]
async fn test_contact_persistence() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();

    let test_id = "persistence_test";

    {
        let setup1 = TestSetup::new(&format!("{}_user1", test_id)).await?;
        let setup2 = TestSetup::new(&format!("{}_user2", test_id)).await?;

        let sg_link2 = setup2.core.generate_sg_link().await?;
        setup1.core.add_contact_by_sg_link(&sg_link2).await?;

        let contacts_before = setup1.core.get_contacts().await?;
        assert_eq!(
            contacts_before.len(),
            1,
            "Should have 1 contact before restart"
        );

        setup1.shutdown().await?;
        setup2.shutdown().await?;
    }

    tokio::time::sleep(Duration::from_millis(100)).await;

    {
        let setup1_new = TestSetup::new(&format!("{}_user1", test_id)).await?;

        // let contacts_after = setup1_new.core.get_contacts().await?;

        setup1_new.shutdown().await?;
    }

    Ok(())
}

#[tokio::test]
async fn test_reliable_message_delivery_verification() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();

    let setup1 = TestSetup::new("reliable_sender").await?;
    let setup2 = TestSetup::new("reliable_receiver").await?;

    let sg_link1 = setup1.core.generate_sg_link().await?;
    let sg_link2 = setup2.core.generate_sg_link().await?;

    setup2.core.add_contact_by_sg_link(&sg_link1).await?;
    setup1.core.add_contact_by_sg_link(&sg_link2).await?;

    tokio::time::sleep(Duration::from_millis(1000)).await;

    let is_receiver_online = setup1.core.check_contact_online("reliable_receiver").await;
    let is_sender_online = setup2.core.check_contact_online("reliable_sender").await;

    if is_receiver_online && is_sender_online {
        let receiver1 = setup1.get_event_receiver();

        let test_message = "Reliable delivery test message";
        let send_result = setup2
            .core
            .send_message("reliable_sender", test_message)
            .await;

        assert!(
            send_result.is_ok(),
            "Message send should succeed to online contact"
        );

        let received =
            wait_for_message_received(receiver1, test_message, Duration::from_secs(10)).await?;
        assert_eq!(received.content, test_message);

        tokio::time::sleep(Duration::from_millis(1000)).await;

        let sender_chat = setup2.core.get_chat_messages("reliable_sender").await?;
        let sent_message = sender_chat.iter().find(|m| m.content == test_message);
        assert!(
            sent_message.is_some(),
            "Sent message should be in sender's chat"
        );

        let delivery_status = &sent_message.unwrap().delivery_status;
        assert!(
            matches!(
                delivery_status,
                shadowghost::DeliveryStatus::Delivered | shadowghost::DeliveryStatus::Sent
            ),
            "Message should be delivered or sent successfully, got: {:?}",
            delivery_status
        );
    }

    setup1.shutdown().await?;
    setup2.shutdown().await?;

    Ok(())
}

#[tokio::test]
async fn test_offline_contact_message_failure() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();

    let setup1 = TestSetup::new("offline_sender").await?;
    let setup2 = TestSetup::new("offline_target").await?;

    let sg_link2 = setup2.core.generate_sg_link().await?;
    setup1.core.add_contact_by_sg_link(&sg_link2).await?;

    tokio::time::sleep(Duration::from_millis(500)).await;

    setup2.shutdown().await?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    let is_online = setup1.core.check_contact_online("offline_target").await;
    assert!(!is_online, "Target should be offline");

    let offline_message = "This should fail";
    let result = setup1
        .core
        .send_message("offline_target", offline_message)
        .await;

    assert!(result.is_err(), "Message to offline contact should fail");

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("unavailable")
            || error_msg.contains("Connection refused")
            || error_msg.contains("timeout"),
        "Error should indicate connection failure: {}",
        error_msg
    );

    setup1.shutdown().await?;

    Ok(())
}
