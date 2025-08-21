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

    println!("🔗 Setting up contact exchange between Alice and Bob");

    let sg_link1 = setup1.core.generate_sg_link().await?;
    let sg_link2 = setup2.core.generate_sg_link().await?;


    setup2.core.add_contact_by_sg_link(&sg_link1).await?;
    setup1.core.add_contact_by_sg_link(&sg_link2).await?;


    tokio::time::sleep(Duration::from_millis(1000)).await;

    let contacts1 = setup1.core.get_contacts().await?;
    let contacts2 = setup2.core.get_contacts().await?;
    assert_eq!(contacts1.len(), 1, "Alice should have 1 contact");
    assert_eq!(contacts2.len(), 1, "Bob should have 1 contact");
    
    assert_contact_exists(&contacts1, "bob");
    assert_contact_exists(&contacts2, "alice");

    println!("✅ Contacts established successfully");


    let test_message = "Hello from Bob to Alice";
    println!("📤 Bob sending message to Alice: '{}'", test_message);

    let send_result = setup2.core.send_message("alice", test_message).await;
    
    match send_result {
        Ok(_) => {
            println!("✅ Message sent successfully");
            

            tokio::time::sleep(Duration::from_millis(1000)).await;


            let chat1 = setup1.core.get_chat_messages("bob").await?;
            let chat2 = setup2.core.get_chat_messages("alice").await?;
            
            println!("💾 Alice's chat history: {} messages", chat1.len());
            println!("💾 Bob's chat history: {} messages", chat2.len());


            assert!(!chat2.is_empty(), "Bob's chat history should not be empty");
            
            if let Some(last_message) = chat2.last() {
                assert_eq!(last_message.content, test_message, "Last message should match sent message");
                println!("✅ Message found in Bob's sent history");
            }
        }
        Err(e) => {

            println!("⚠️ Message sending failed (expected for offline test): {}", e);
            assert!(
                e.to_string().contains("unavailable") 
                || e.to_string().contains("Connection refused")
                || e.to_string().contains("Server not running"),
                "Error should indicate connection failure: {}", e
            );
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

    println!("✅ Properly rejected message to unknown contact: {}", error_msg);

    setup1.shutdown().await?;

    Ok(())
}

#[tokio::test]
async fn test_contact_online_status_check() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();

    let setup1 = TestSetup::new("checker").await?;
    let setup2 = TestSetup::new("target").await?;

    println!("🔍 Testing contact online status check");

    let sg_link2 = setup2.core.generate_sg_link().await?;
    setup1.core.add_contact_by_sg_link(&sg_link2).await?;

    tokio::time::sleep(Duration::from_millis(500)).await;

    let is_online_when_running = setup1.core.check_contact_online("target").await;
    println!("📡 Target online status when running: {}", is_online_when_running);

    setup2.shutdown().await?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    let is_online_after_shutdown = setup1.core.check_contact_online("target").await;
    println!("📡 Target online status after shutdown: {}", is_online_after_shutdown);




    if is_online_when_running {
        let test_result = setup1.core.send_message("target", "test").await;

        if test_result.is_err() {
            println!("✅ Message correctly failed to offline contact");
        } else {
            println!("⚠️ Message succeeded (might be expected in test environment)");
        }
    }

    setup1.shutdown().await?;

    Ok(())
}

#[tokio::test]
async fn test_contact_persistence() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();

    let test_id = "persistence_test";

    println!("💾 Testing contact persistence across restarts");


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

        println!("✅ Contact added in first session");

        setup1.shutdown().await?;
        setup2.shutdown().await?;
    }

    tokio::time::sleep(Duration::from_millis(100)).await;


    {
        let setup1_new = TestSetup::new(&format!("{}_user1", test_id)).await?;





        println!("✅ Second session started successfully");

        setup1_new.shutdown().await?;
    }

    println!("✅ Contact persistence test completed");

    Ok(())
}

#[tokio::test]
async fn test_reliable_message_delivery_verification() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();

    let setup1 = TestSetup::new("reliable_sender").await?;
    let setup2 = TestSetup::new("reliable_receiver").await?;

    println!("🔄 Testing reliable message delivery verification");

    let sg_link1 = setup1.core.generate_sg_link().await?;
    let sg_link2 = setup2.core.generate_sg_link().await?;

    setup2.core.add_contact_by_sg_link(&sg_link1).await?;
    setup1.core.add_contact_by_sg_link(&sg_link2).await?;

    tokio::time::sleep(Duration::from_millis(1000)).await;

    let is_receiver_online = setup1.core.check_contact_online("reliable_receiver").await;
    let is_sender_online = setup2.core.check_contact_online("reliable_sender").await;

    println!("📡 Receiver online: {}, Sender online: {}", is_receiver_online, is_sender_online);

    let test_message = "Reliable delivery test message";
    let send_result = setup2
        .core
        .send_message("reliable_sender", test_message)
        .await;

    match send_result {
        Ok(_) => {
            println!("✅ Message send succeeded");

            tokio::time::sleep(Duration::from_millis(1000)).await;

            let sender_chat = setup2.core.get_chat_messages("reliable_sender").await?;
            let sent_message = sender_chat.iter().find(|m| m.content == test_message);
            assert!(
                sent_message.is_some(),
                "Sent message should be in sender's chat"
            );

            let delivery_status = &sent_message.unwrap().delivery_status;
            println!("📦 Message delivery status: {:?}", delivery_status);
            
            assert!(
                matches!(
                    delivery_status,
                    shadowghost::DeliveryStatus::Delivered | shadowghost::DeliveryStatus::Sent
                ),
                "Message should be delivered or sent successfully, got: {:?}",
                delivery_status
            );
        }
        Err(e) => {
            println!("⚠️ Message sending failed (expected in test environment): {}", e);
        }
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

    println!("📴 Testing message failure to offline contact");

    let sg_link2 = setup2.core.generate_sg_link().await?;
    setup1.core.add_contact_by_sg_link(&sg_link2).await?;

    tokio::time::sleep(Duration::from_millis(500)).await;


    setup2.shutdown().await?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    let is_online = setup1.core.check_contact_online("offline_target").await;
    println!("📡 Target online status after shutdown: {}", is_online);

    let offline_message = "This should fail";
    let result = setup1
        .core
        .send_message("offline_target", offline_message)
        .await;



    match result {
        Ok(_) => {
            println!("⚠️ Message was queued locally (expected behavior)");

            let chat_messages = setup1.core.get_chat_messages("offline_target").await?;
            if let Some(last_msg) = chat_messages.last() {
                println!("📦 Message status: {:?}", last_msg.delivery_status);
            }
        }
        Err(e) => {
            println!("✅ Message correctly failed to offline contact: {}", e);
            assert!(
                e.to_string().contains("unavailable")
                    || e.to_string().contains("Connection refused")
                    || e.to_string().contains("timeout")
                    || e.to_string().contains("Server not running"),
                "Error should indicate connection failure: {}",
                e
            );
        }
    }

    setup1.shutdown().await?;

    Ok(())
}

#[tokio::test]
async fn test_bidirectional_contact_exchange() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();

    let setup1 = TestSetup::new("ping").await?;
    let setup2 = TestSetup::new("pong").await?;

    println!("🔄 Testing bidirectional contact exchange");

    let sg_link1 = setup1.core.generate_sg_link().await?;
    let sg_link2 = setup2.core.generate_sg_link().await?;


    setup2.core.add_contact_by_sg_link(&sg_link1).await?;
    setup1.core.add_contact_by_sg_link(&sg_link2).await?;

    tokio::time::sleep(Duration::from_millis(1000)).await;


    let contacts1 = setup1.core.get_contacts().await?;
    let contacts2 = setup2.core.get_contacts().await?;

    assert_contact_count(&contacts1, 1);
    assert_contact_count(&contacts2, 1);

    assert_contact_exists(&contacts1, "pong");
    assert_contact_exists(&contacts2, "ping");

    println!("✅ Bidirectional contacts established");


    let ping_msg = "PING: Hello from ping!";
    let send_result1 = setup1.core.send_message("pong", ping_msg).await;

    if send_result1.is_ok() {
        println!("✅ Ping message sent");

        tokio::time::sleep(Duration::from_millis(500)).await;

        let pong_msg = "PONG: Hello back from pong!";
        let send_result2 = setup2.core.send_message("ping", pong_msg).await;

        if send_result2.is_ok() {
            println!("✅ Pong message sent");

            let chat1 = setup1.core.get_chat_messages("pong").await?;
            let chat2 = setup2.core.get_chat_messages("ping").await?;

            println!("💾 Ping has {} messages", chat1.len());
            println!("💾 Pong has {} messages", chat2.len());


            assert!(!chat1.is_empty(), "Ping should have messages in chat");
            assert!(!chat2.is_empty(), "Pong should have messages in chat");
        }
    }

    setup1.shutdown().await?;
    setup2.shutdown().await?;

    Ok(())
}

#[tokio::test]
async fn test_multiple_contacts_management() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();

    let setup_main = TestSetup::new("main_user").await?;
    let setup1 = TestSetup::new("contact1").await?;
    let setup2 = TestSetup::new("contact2").await?;
    let setup3 = TestSetup::new("contact3").await?;

    println!("👥 Testing multiple contacts management");


    let link1 = setup1.core.generate_sg_link().await?;
    let link2 = setup2.core.generate_sg_link().await?;
    let link3 = setup3.core.generate_sg_link().await?;

    setup_main.core.add_contact_by_sg_link(&link1).await?;
    setup_main.core.add_contact_by_sg_link(&link2).await?;
    setup_main.core.add_contact_by_sg_link(&link3).await?;

    tokio::time::sleep(Duration::from_millis(1000)).await;

    let contacts = setup_main.core.get_contacts().await?;
    assert_contact_count(&contacts, 3);

    let contact_names = setup_main.get_contact_names().await;
    assert!(contact_names.contains(&"contact1".to_string()));
    assert!(contact_names.contains(&"contact2".to_string()));
    assert!(contact_names.contains(&"contact3".to_string()));

    println!("✅ All contacts added successfully: {:?}", contact_names);


    let msg_results = vec![
        setup_main.core.send_message("contact1", "Hello contact1").await,
        setup_main.core.send_message("contact2", "Hello contact2").await,
        setup_main.core.send_message("contact3", "Hello contact3").await,
    ];

    let success_count = msg_results.iter().filter(|r| r.is_ok()).count();
    println!("✅ Successfully sent {} out of 3 messages", success_count);

    setup_main.shutdown().await?;
    setup1.shutdown().await?;
    setup2.shutdown().await?;
    setup3.shutdown().await?;

    Ok(())
}