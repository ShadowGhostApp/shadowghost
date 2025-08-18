mod common;
#[path = "common/events.rs"]
mod events;
use common::*;
use events::*;
use std::time::Duration;

#[tokio::test]
async fn test_message_sending_and_receiving() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();

    let mut setup1 = TestSetup::new("1").await?;
    let mut setup2 = TestSetup::new("2").await?;

    let sg_link1 = setup1.core.generate_sg_link().await?;
    let sg_link2 = setup2.core.generate_sg_link().await?;

    setup2.core.add_contact_by_sg_link(&sg_link1).await?;
    setup1.core.add_contact_by_sg_link(&sg_link2).await?;

    tokio::time::sleep(Duration::from_millis(1000)).await;

    let contacts1 = setup1.core.get_contacts().await?;
    let contacts2 = setup2.core.get_contacts().await?;
    assert_eq!(contacts1.len(), 1);
    assert_eq!(contacts2.len(), 1);
    assert_eq!(contacts1[0].name, "2");
    assert_eq!(contacts2[0].name, "1");

    let receiver1 = setup1.get_event_receiver();

    let msg = "hello from 2 to 1";
    setup2.core.send_message("1", msg).await?;

    let received = wait_for_message_received(receiver1, msg, Duration::from_secs(10)).await?;
    assert_eq!(received.content, msg);

    let chat1 = setup1.core.get_chat_messages("2").await?;
    assert!(!chat1.is_empty());
    assert_eq!(chat1.last().unwrap().content, msg);

    let chat2 = setup2.core.get_chat_messages("1").await?;
    assert!(!chat2.is_empty());

    setup1.shutdown().await?;
    setup2.shutdown().await?;

    Ok(())
}

#[tokio::test]
async fn test_multiple_message_exchange() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();

    let mut setup1 = TestSetup::new("1").await?;
    let mut setup2 = TestSetup::new("2").await?;

    let sg_link1 = setup1.core.generate_sg_link().await?;
    let sg_link2 = setup2.core.generate_sg_link().await?;

    setup2.core.add_contact_by_sg_link(&sg_link1).await?;
    setup1.core.add_contact_by_sg_link(&sg_link2).await?;

    tokio::time::sleep(Duration::from_millis(1000)).await;

    let messages = vec![
        ("2", "1", "m1"),
        ("1", "2", "m2"),
        ("2", "1", "m3"),
        ("1", "2", "m4"),
    ];

    for (sender, recipient, content) in &messages {
        let setup = if *sender == "1" { &setup1 } else { &setup2 };
        setup.core.send_message(recipient, content).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    tokio::time::sleep(Duration::from_millis(2000)).await;

    let chat1 = setup1.core.get_chat_messages("2").await?;
    let chat2 = setup2.core.get_chat_messages("1").await?;

    assert!(chat1.len() >= 2);
    assert!(chat2.len() >= 2);

    setup1.shutdown().await?;
    setup2.shutdown().await?;

    Ok(())
}

#[tokio::test]
async fn test_message_to_unknown_contact() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();

    let mut setup1 = TestSetup::new("1").await?;

    let result = setup1.core.send_message("99", "hi unknown").await;
    assert!(result.is_err());

    let err = result.unwrap_err().to_string();
    assert!(err.contains("not found") || err.contains("Contact"));

    setup1.shutdown().await?;

    Ok(())
}
