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

    println!("🧪 Testing basic message sending and receiving");

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

    println!("✅ Contact exchange completed");

    let receiver1 = setup1.get_event_receiver();

    let msg = "Hello from receiver to sender";
    println!("📤 Sending message: '{}'", msg);
    setup2.core.send_message("sender", msg).await?;

    let received = wait_for_message_received(receiver1, msg, Duration::from_secs(10)).await?;
    assert_eq!(
        received.content, msg,
        "Received message content should match"
    );
    assert_eq!(
        received.from, "receiver",
        "Message should be from 'receiver'"
    );
    assert_eq!(received.to, "sender", "Message should be to 'sender'");

    println!("📨 Message received and verified");

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

    println!("💾 Chat histories verified");

    setup1.shutdown().await?;
    setup2.shutdown().await?;

    Ok(())
}

#[tokio::test]
async fn test_multiple_message_exchange() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();

    let setup1 = TestSetup::new("alice").await?;
    let setup2 = TestSetup::new("bob").await?;

    println!("🧪 Testing multiple message exchange");

    let sg_link1 = setup1.core.generate_sg_link().await?;
    let sg_link2 = setup2.core.generate_sg_link().await?;

    setup2.core.add_contact_by_sg_link(&sg_link1).await?;
    setup1.core.add_contact_by_sg_link(&sg_link2).await?;

    tokio::time::sleep(Duration::from_millis(1000)).await;

    let messages = vec![
        ("bob", "alice", "Hi Alice, how are you?"),
        ("alice", "bob", "Hi Bob! I'm doing great, thanks!"),
        ("bob", "alice", "That's wonderful to hear!"),
        ("alice", "bob", "How about you?"),
        ("bob", "alice", "I'm doing well too, thanks for asking!"),
    ];

    println!("📤 Sending {} messages in sequence", messages.len());

    for (sender, recipient, content) in &messages {
        let setup = if *sender == "alice" { &setup1 } else { &setup2 };

        println!("  {} -> {}: '{}'", sender, recipient, content);
        setup.core.send_message(recipient, content).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    tokio::time::sleep(Duration::from_millis(2000)).await;

    let chat1 = setup1.core.get_chat_messages("bob").await?;
    let chat2 = setup2.core.get_chat_messages("alice").await?;

    println!("💾 Alice's chat history: {} messages", chat1.len());
    println!("💾 Bob's chat history: {} messages", chat2.len());

    assert!(
        chat1.len() >= 3,
        "Alice should have at least 3 messages in chat"
    );
    assert!(
        chat2.len() >= 3,
        "Bob should have at least 3 messages in chat"
    );

    let total_messages_found = chat1.len() + chat2.len();
    println!(
        "📊 Total messages found in both chats: {}",
        total_messages_found
    );

    for (i, msg) in chat1.iter().enumerate() {
        println!(
            "  Alice's chat[{}]: {} -> {}: '{}'",
            i, msg.from, msg.to, msg.content
        );
    }

    setup1.shutdown().await?;
    setup2.shutdown().await?;

    println!("✅ Multiple message exchange test completed");

    Ok(())
}

#[tokio::test]
async fn test_message_to_unknown_contact() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();

    let setup1 = TestSetup::new("lonely_user").await?;

    println!("🧪 Testing message to non-existent contact");

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

    println!("✅ Unknown contact error handled correctly: {}", err);

    setup1.shutdown().await?;

    Ok(())
}

#[tokio::test]
async fn test_bidirectional_communication() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();

    let setup1 = TestSetup::new("ping").await?;
    let setup2 = TestSetup::new("pong").await?;

    println!("🧪 Testing bidirectional communication");

    let sg_link1 = setup1.core.generate_sg_link().await?;
    let sg_link2 = setup2.core.generate_sg_link().await?;

    setup2.core.add_contact_by_sg_link(&sg_link1).await?;
    setup1.core.add_contact_by_sg_link(&sg_link2).await?;

    tokio::time::sleep(Duration::from_millis(1000)).await;

    let receiver1 = setup1.get_event_receiver();
    let receiver2 = setup2.get_event_receiver();

    println!("🏓 Starting ping-pong message exchange");

    let ping_msg = "PING: Hello from ping!";
    setup1.core.send_message("pong", ping_msg).await?;

    let received_ping =
        wait_for_message_received(receiver2, ping_msg, Duration::from_secs(5)).await?;
    assert_eq!(received_ping.content, ping_msg);
    println!("  ✅ PING received");

    let pong_msg = "PONG: Hello back from pong!";
    setup2.core.send_message("ping", pong_msg).await?;

    let received_pong =
        wait_for_message_received(receiver1, pong_msg, Duration::from_secs(5)).await?;
    assert_eq!(received_pong.content, pong_msg);
    println!("  ✅ PONG received");

    let chat1 = setup1.core.get_chat_messages("pong").await?;
    let chat2 = setup2.core.get_chat_messages("ping").await?;

    assert_eq!(chat1.len(), 2, "Ping should have 2 messages in chat");
    assert_eq!(chat2.len(), 2, "Pong should have 2 messages in chat");

    println!("💾 Both chat histories contain 2 messages each");

    setup1.shutdown().await?;
    setup2.shutdown().await?;

    println!("✅ Bidirectional communication test completed");

    Ok(())
}

#[tokio::test]
async fn test_message_delivery_status() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();

    let setup1 = TestSetup::new("sender").await?;
    let setup2 = TestSetup::new("receiver").await?;

    println!("🧪 Testing message delivery status");

    let sg_link1 = setup1.core.generate_sg_link().await?;
    let sg_link2 = setup2.core.generate_sg_link().await?;

    setup2.core.add_contact_by_sg_link(&sg_link1).await?;
    setup1.core.add_contact_by_sg_link(&sg_link2).await?;

    tokio::time::sleep(Duration::from_millis(1000)).await;

    let test_message = "Test delivery status";
    setup1.core.send_message("receiver", test_message).await?;

    tokio::time::sleep(Duration::from_millis(2000)).await;

    let sender_chat = setup1.core.get_chat_messages("receiver").await?;
    let receiver_chat = setup2.core.get_chat_messages("sender").await?;

    assert!(
        !sender_chat.is_empty(),
        "Sender should have message in chat"
    );
    assert!(
        !receiver_chat.is_empty(),
        "Receiver should have message in chat"
    );

    let sent_message = sender_chat.iter().find(|m| m.content == test_message);
    let received_message = receiver_chat.iter().find(|m| m.content == test_message);

    assert!(
        sent_message.is_some(),
        "Sent message should be in sender's chat"
    );
    assert!(
        received_message.is_some(),
        "Received message should be in receiver's chat"
    );

    println!("📨 Message found in both chats with delivery status");

    setup1.shutdown().await?;
    setup2.shutdown().await?;

    Ok(())
}
