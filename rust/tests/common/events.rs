use crate::common::TestSetup;
use shadowghost::{AppEvent, ChatMessage, NetworkEvent};
use std::error::Error;
use std::time::Duration;
use tokio::sync::broadcast::Receiver;
use tokio::time::timeout;

impl TestSetup {
    pub fn get_event_receiver(&self) -> Receiver<AppEvent> {
        self.core.get_event_bus().subscribe()
    }
}

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
