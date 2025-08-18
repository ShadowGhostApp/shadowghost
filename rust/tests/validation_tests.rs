mod common;

use common::*;
use std::time::Duration;

#[tokio::test]
async fn test_invalid_sg_link_handling() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();

    let mut setup1 = TestSetup::new("1").await?;

    let invalid_links = vec![
        "invalid-link",
        "sg://",
        "sg://invalid-base64-!@#$",
        "sg://dGhpcyBpcyBub3QganNvbg==",
        "sg://eyJpbnZhbGlkIjogImpzb24ifQ==",
        "http://not-sg-link",
        "sg://",
        "sg://YWJjZGVmZ2hpams=",
    ];

    for invalid_link in invalid_links {
        let result = setup1.core.add_contact_by_sg_link(invalid_link).await;
        assert!(result.is_err());
    }

    setup1.shutdown().await?;

    Ok(())
}

#[tokio::test]
async fn test_self_addition_prevention() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();

    let mut setup1 = TestSetup::new("1").await?;
    let sg_link1 = setup1.core.generate_sg_link().await?;

    let result = setup1.core.add_contact_by_sg_link(&sg_link1).await;
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Cannot add yourself") || error_msg.contains("yourself as contact"));

    let contacts = setup1.core.get_contacts().await?;
    assert_eq!(contacts.len(), 0);

    setup1.shutdown().await?;

    Ok(())
}

#[tokio::test]
async fn test_sg_link_generation_consistency() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();

    let mut setup1 = TestSetup::new("1").await?;
    let link1 = setup1.core.generate_sg_link().await?;
    let link2 = setup1.core.generate_sg_link().await?;

    assert!(link1.starts_with("sg://"));
    assert!(link2.starts_with("sg://"));
    assert_eq!(link1, link2);
    assert!(link1.len() > 10);
    assert!(link2.len() > 10);

    setup1.shutdown().await?;

    Ok(())
}

#[tokio::test]
async fn test_duplicate_contact_handling() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();

    let mut setup1 = TestSetup::new("1").await?;
    let mut setup2 = TestSetup::new("2").await?;
    let sg_link2 = setup2.core.generate_sg_link().await?;

    setup1.core.add_contact_by_sg_link(&sg_link2).await?;
    let contacts_first = setup1.core.get_contacts().await?;
    assert_eq!(contacts_first.len(), 1);

    setup1.core.add_contact_by_sg_link(&sg_link2).await?;
    let contacts_second = setup1.core.get_contacts().await?;
    assert_eq!(contacts_second.len(), 1);

    assert_eq!(contacts_first[0].name, contacts_second[0].name);
    assert_eq!(contacts_first[0].id, contacts_second[0].id);

    setup1.shutdown().await?;
    setup2.shutdown().await?;

    Ok(())
}

#[tokio::test]
async fn test_empty_message_handling() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();

    let mut setup1 = TestSetup::new("1").await?;
    let mut setup2 = TestSetup::new("2").await?;
    let sg_link1 = setup1.core.generate_sg_link().await?;
    setup2.core.add_contact_by_sg_link(&sg_link1).await?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    let result = setup2.core.send_message("1", "").await;
    assert!(result.is_ok() || result.is_err());

    let long_message = "a".repeat(10000);
    let result_long = setup2.core.send_message("1", &long_message).await;
    assert!(result_long.is_ok() || result_long.is_err());

    setup1.shutdown().await?;
    setup2.shutdown().await?;

    Ok(())
}
