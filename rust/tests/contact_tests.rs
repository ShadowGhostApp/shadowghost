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

    let test_message = "Hello from 2";
    setup2.core.send_message("1", test_message).await?;

    let received =
        wait_for_message_received(receiver1, test_message, Duration::from_secs(10)).await?;
    assert_eq!(received.content, test_message);

    let chat1 = setup1.core.get_chat_messages("2").await?;
    let chat2 = setup2.core.get_chat_messages("1").await?;
    assert_eq!(chat1.last().unwrap().content, test_message);
    assert_eq!(chat2.last().unwrap().content, test_message);

    setup1.shutdown().await?;
    setup2.shutdown().await?;

    Ok(())
}

#[tokio::test]
async fn test_message_to_unknown_contact() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();

    let mut setup1 = TestSetup::new("1").await?;

    let result = setup1.core.send_message("999", "Hello unknown").await;
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("not found") || error_msg.contains("Contact"));

    setup1.shutdown().await?;

    Ok(())
}
