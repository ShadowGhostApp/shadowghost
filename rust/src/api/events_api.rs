use super::core_api::CORE;
use crate::events::bus::AppEvent;
use crate::frb_generated::StreamSink;
use flutter_rust_bridge::frb;
use std::collections::VecDeque;
use std::sync::{LazyLock, Mutex};

static EVENT_QUEUE: LazyLock<Mutex<VecDeque<AppEvent>>> =
    LazyLock::new(|| Mutex::new(VecDeque::new()));

#[frb]
pub async fn listen_to_events(sink: StreamSink<AppEvent>) -> Result<(), String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.clone() {
        drop(core_guard);

        tokio::spawn(async move {
            let mut event_receiver = core.lock().await.get_event_bus().subscribe();

            while let Ok(event) = event_receiver.recv().await {
                // Send event to Flutter through StreamSink
                let _ = sink.add(event.clone());

                // Also save to queue for compatibility
                let mut queue = EVENT_QUEUE.lock().unwrap();
                queue.push_back(event);
                if queue.len() > 1000 {
                    queue.pop_front();
                }
            }
        });

        Ok(())
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb]
pub async fn start_event_listener() -> Result<String, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.clone() {
        drop(core_guard);

        tokio::spawn(async move {
            let mut event_receiver = core.lock().await.get_event_bus().subscribe();
            while let Ok(event) = event_receiver.recv().await {
                let mut queue = EVENT_QUEUE.lock().unwrap();
                queue.push_back(event);
                if queue.len() > 1000 {
                    queue.pop_front();
                }
            }
        });

        Ok("Event listener started".to_string())
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb]
pub async fn get_pending_events() -> Result<Vec<AppEvent>, String> {
    let mut queue = EVENT_QUEUE.lock().unwrap();
    let events: Vec<AppEvent> = queue.drain(..).collect();
    Ok(events)
}

#[frb]
pub async fn has_pending_events() -> Result<bool, String> {
    let queue = EVENT_QUEUE.lock().unwrap();
    Ok(!queue.is_empty())
}

#[frb]
pub async fn clear_event_queue() -> Result<String, String> {
    let mut queue = EVENT_QUEUE.lock().unwrap();
    queue.clear();
    Ok("Event queue cleared".to_string())
}

#[frb]
pub async fn emit_custom_event(event_type: String, data: String) -> Result<String, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.clone() {
        drop(core_guard);

        let custom_event = AppEvent::Custom {
            event_type: event_type.clone(),
            data: data.clone(),
            timestamp: chrono::Utc::now().timestamp() as u64,
        };

        core.lock().await.get_event_bus().emit(custom_event);
        Ok(format!("Custom event '{}' emitted: {}", event_type, data))
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb]
pub async fn get_event_queue_size() -> Result<u32, String> {
    let queue = EVENT_QUEUE.lock().unwrap();
    Ok(queue.len() as u32)
}

#[frb]
pub async fn subscribe_to_message_events(sink: StreamSink<AppEvent>) -> Result<(), String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.clone() {
        drop(core_guard);

        tokio::spawn(async move {
            let mut event_receiver = core.lock().await.get_event_bus().subscribe();

            while let Ok(event) = event_receiver.recv().await {
                // Filter only message events
                if let AppEvent::Network(crate::events::NetworkEvent::MessageReceived { .. }) =
                    &event
                {
                    let _ = sink.add(event);
                }
            }
        });

        Ok(())
    } else {
        Err("Core not initialized".to_string())
    }
}
