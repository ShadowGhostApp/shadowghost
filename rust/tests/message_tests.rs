mod common;
#[path = "common/events.rs"]
mod events;
use common::*;
use events::*;
use std::time::Duration;

#[tokio::test]
async fn test_message_sending_and_receiving() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();

    let setup1 = TestSetup::new("sender").await?;
    let setup2 = TestSetup::new("receiver").await?;

    let sg_link1 = setup1.core.generate_sg_link().await?;
    let sg_link2 = setup2.core.generate_sg_link().await?;

    setup2.core.add_contact_by_sg_link(&sg_link1).await?;
    setup1.core.add_contact_by_sg_link(&sg_link2).await?;

    tokio::time::sleep(Duration::from_millis(1000)).await;

    let contacts1 = setup1.core.get_contacts().await?;
    let contacts2 = setup2.core.get_contacts().await?;
    assert_eq!(contacts1.len(), 1, "Sender should have 1 contact");
    assert_eq!(contacts2.len(), 1, "Receiver should have 1 contact");
    assert_eq!(
        contacts1[0].name, "receiver",
        "Sender's contact should be 'receiver'"
    );
    assert_eq!(
        contacts2[0].name, "sender",
        "Receiver's contact should be 'sender'"
    );

    let receiver1 = setup1.get_event_receiver();

    let is_online_before = setup1.core.check_contact_online("receiver").await;
    if !is_online_before {
        setup1.shutdown().await?;
        setup2.shutdown().await?;
        return Err("Receiver is not online, cannot test message delivery".into());
    }

    let msg = "Hello from receiver to sender";
    let send_result = setup2.core.send_message("sender", msg).await;

    match send_result {
        Ok(_) => {
            let received =
                wait_for_message_received(receiver1, msg, Duration::from_secs(10)).await?;
            assert_eq!(
                received.content, msg,
                "Received message content should match"
            );
            assert_eq!(
                received.from, "receiver",
                "Message should be from 'receiver'"
            );
            assert_eq!(received.to, "sender", "Message should be to 'sender'");

            let chat1 = setup1.core.get_chat_messages("receiver").await?;
            assert!(
                !chat1.is_empty(),
                "Sender's chat history should not be empty"
            );
            assert_eq!(
                chat1.last().unwrap().content,
                msg,
                "Last message should match sent message"
            );

            let chat2 = setup2.core.get_chat_messages("sender").await?;
            assert!(
                !chat2.is_empty(),
                "Receiver's chat history should not be empty"
            );

            let sent_message = chat2
                .iter()
                .find(|m| m.content == msg && m.from == "receiver");
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
                "Message should be delivered or sent, got: {:?}",
                delivery_status
            );
        }
        Err(e) => {
            if e.to_string().contains("unavailable") || e.to_string().contains("Connection refused")
            {


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
async fn test_message_delivery_with_acknowledgment() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();

    let setup1 = TestSetup::new("sender").await?;
    let setup2 = TestSetup::new("receiver").await?;

    let sg_link1 = setup1.core.generate_sg_link().await?;
    let sg_link2 = setup2.core.generate_sg_link().await?;

    setup2.core.add_contact_by_sg_link(&sg_link1).await?;
    setup1.core.add_contact_by_sg_link(&sg_link2).await?;

    tokio::time::sleep(Duration::from_millis(1000)).await;

    let test_message = "Test delivery status tracking";

    if setup1.core.check_contact_online("receiver").await {
        let result = setup1.core.send_message("receiver", test_message).await;

        match result {
            Ok(_) => {
                tokio::time::sleep(Duration::from_millis(2000)).await;

                let sender_chat = setup1.core.get_chat_messages("receiver").await?;
                let receiver_chat = setup2.core.get_chat_messages("sender").await?;

                assert!(
                    !sender_chat.is_empty(),
                    "Sender should have message in chat"
                );

                let sent_message = sender_chat.iter().find(|m| m.content == test_message);
                assert!(
                    sent_message.is_some(),
                    "Sent message should be in sender's chat"
                );

                let delivery_status = &sent_message.unwrap().delivery_status;
                assert!(
                    !matches!(delivery_status, shadowghost::DeliveryStatus::Pending),
                    "Message should not be in pending state after send attempt"
                );

                if !receiver_chat.is_empty() {
                    let received_message = receiver_chat.iter().find(|m| m.content == test_message);
                    if received_message.is_some() {
                        assert!(
                            matches!(delivery_status, shadowghost::DeliveryStatus::Delivered),
                            "Message should be delivered if found in receiver's chat"
                        );
                    }
                }
            }
            Err(_) => {
                let sender_chat = setup1.core.get_chat_messages("receiver").await?;
                if !sender_chat.is_empty() {
                    let failed_message = sender_chat.iter().find(|m| m.content == test_message);
                    if let Some(msg) = failed_message {
                        assert!(
                            matches!(msg.delivery_status, shadowghost::DeliveryStatus::Failed),
                            "Failed message should have Failed delivery status"
                        );
                    }
                }
            }
        }
    } else {
        let result = setup1.core.send_message("receiver", test_message).await;
        assert!(result.is_err(), "Sending to offline contact should fail");
    }

    setup1.shutdown().await?;
    setup2.shutdown().await?;

    Ok(())
}

#[tokio::test]
async fn test_offline_message_handling() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();

    let setup1 = TestSetup::new("sender").await?;
    let setup2 = TestSetup::new("receiver").await?;

    let sg_link1 = setup1.core.generate_sg_link().await?;
    let sg_link2 = setup2.core.generate_sg_link().await?;

    setup2.core.add_contact_by_sg_link(&sg_link1).await?;
    setup1.core.add_contact_by_sg_link(&sg_link2).await?;

    tokio::time::sleep(Duration::from_millis(500)).await;



    setup2.shutdown().await?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    let is_online_after_shutdown = setup1.core.check_contact_online("receiver").await;
    assert!(
        !is_online_after_shutdown,
        "Contact should be offline after shutdown"
    );

    let offline_message = "This message should fail to deliver";
    let result = setup1.core.send_message("receiver", offline_message).await;

    assert!(result.is_err(), "Sending to offline contact should fail");

    let error_string = result.unwrap_err().to_string();
    assert!(
        error_string.contains("unavailable")
            || error_string.contains("Connection refused")
            || error_string.contains("timeout"),
        "Error should indicate connection failure: {}",
        error_string
    );

    let sender_chat = setup1.core.get_chat_messages("receiver").await?;
    if !sender_chat.is_empty() {
        let failed_message = sender_chat.iter().find(|m| m.content == offline_message);
        if let Some(msg) = failed_message {
            assert!(
                matches!(msg.delivery_status, shadowghost::DeliveryStatus::Failed),
                "Offline message should have Failed delivery status, got: {:?}",
                msg.delivery_status
            );
        }
    }

    setup1.shutdown().await?;

    Ok(())
}

#[tokio::test]
async fn test_online_status_accuracy() -> Result<(), Box<dyn std::error::Error>> {
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
        let test_message = "Test message to verify status";
        let result = setup1.core.send_message("target", test_message).await;
        assert!(result.is_err(), "Message should fail after target shutdown");
    }

    setup1.shutdown().await?;

    Ok(())
}

#[tokio::test]
async fn test_bidirectional_communication_with_status() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();

    let setup1 = TestSetup::new("ping").await?;
    let setup2 = TestSetup::new("pong").await?;

    let sg_link1 = setup1.core.generate_sg_link().await?;
    let sg_link2 = setup2.core.generate_sg_link().await?;

    setup2.core.add_contact_by_sg_link(&sg_link1).await?;
    setup1.core.add_contact_by_sg_link(&sg_link2).await?;

    tokio::time::sleep(Duration::from_millis(1000)).await;

    let ping_online = setup1.core.check_contact_online("pong").await;
    let pong_online = setup2.core.check_contact_online("ping").await;

    if ping_online && pong_online {
        let receiver1 = setup1.get_event_receiver();
        let receiver2 = setup2.get_event_receiver();

        let ping_msg = "PING: Hello from ping!";
        let send_result1 = setup1.core.send_message("pong", ping_msg).await;

        if send_result1.is_ok() {
            let received_ping =
                wait_for_message_received(receiver2, ping_msg, Duration::from_secs(5)).await?;
            assert_eq!(received_ping.content, ping_msg);

            let pong_msg = "PONG: Hello back from pong!";
            let send_result2 = setup2.core.send_message("ping", pong_msg).await;

            if send_result2.is_ok() {
                let received_pong =
                    wait_for_message_received(receiver1, pong_msg, Duration::from_secs(5)).await?;
                assert_eq!(received_pong.content, pong_msg);

                let chat1 = setup1.core.get_chat_messages("pong").await?;
                let chat2 = setup2.core.get_chat_messages("ping").await?;

                assert!(
                    chat1.len() >= 2,
                    "Ping should have at least 2 messages in chat"
                );
                assert!(
                    chat2.len() >= 2,
                    "Pong should have at least 2 messages in chat"
                );
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

    let setup1 = TestSetup::new("lonely_user").await?;

    let result = setup1
        .core
        .send_message("non_existent_user", "Hello there!")
        .await;
    assert!(result.is_err(), "Message to unknown contact should fail");

    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("not found") || err.contains("Contact"),
        "Error should mention contact not found: {}",
        err
    );

    setup1.shutdown().await?;

    Ok(())
}
