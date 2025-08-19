use shadowghost::{AppEvent, ChatMessage, NetworkEvent};
use std::error::Error;
use std::time::Duration;
use tokio::sync::broadcast::Receiver;
use tokio::time::timeout;

pub async fn wait_for_message_received(
    mut receiver: Receiver<AppEvent>,
    expected_content: &str,
    timeout_duration: Duration,
) -> Result<ChatMessage, Box<dyn Error>> {
    let result = timeout(timeout_duration, async {
        loop {
            match receiver.recv().await {
                Ok(AppEvent::Network(NetworkEvent::MessageReceived { message })) => {
                    if message.content == expected_content {
                        return Ok(message);
                    }
                }
                Ok(_) => continue,
                Err(e) => return Err(format!("Event receiver error: {}", e).into()),
            }
        }
    })
    .await;

    match result {
        Ok(message_result) => message_result,
        Err(_) => Err(format!("Timeout waiting for message: {}", expected_content).into()),
    }
}

pub async fn wait_for_contact_added(
    mut receiver: Receiver<AppEvent>,
    expected_name: &str,
    timeout_duration: Duration,
) -> Result<(), Box<dyn Error>> {
    let result = timeout(timeout_duration, async {
        loop {
            match receiver.recv().await {
                Ok(AppEvent::Network(NetworkEvent::ContactAdded { contact })) => {
                    if contact.name == expected_name {
                        return Ok(());
                    }
                }
                Ok(_) => continue,
                Err(e) => return Err(format!("Event receiver error: {}", e).into()),
            }
        }
    })
    .await;

    match result {
        Ok(result) => result,
        Err(_) => Err(format!("Timeout waiting for contact: {}", expected_name).into()),
    }
}

pub async fn wait_for_server_started(
    mut receiver: Receiver<AppEvent>,
    timeout_duration: Duration,
) -> Result<u16, Box<dyn Error>> {
    let result = timeout(timeout_duration, async {
        loop {
            match receiver.recv().await {
                Ok(AppEvent::Network(NetworkEvent::ServerStarted { port })) => {
                    return Ok(port);
                }
                Ok(_) => continue,
                Err(e) => return Err(format!("Event receiver error: {}", e).into()),
            }
        }
    })
    .await;

    match result {
        Ok(port_result) => port_result,
        Err(_) => Err("Timeout waiting for server started event".into()),
    }
}

pub async fn wait_for_error_event(
    mut receiver: Receiver<AppEvent>,
    timeout_duration: Duration,
) -> Result<String, Box<dyn Error>> {
    let result = timeout(timeout_duration, async {
        loop {
            match receiver.recv().await {
                Ok(AppEvent::Network(NetworkEvent::Error { error, context: _ })) => {
                    return Ok(error);
                }
                Ok(_) => continue,
                Err(e) => return Err(format!("Event receiver error: {}", e).into()),
            }
        }
    })
    .await;

    match result {
        Ok(error_result) => error_result,
        Err(_) => Err("Timeout waiting for error event".into()),
    }
}
