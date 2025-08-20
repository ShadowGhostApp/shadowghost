use super::core::CORE;
use crate::prelude::*;
use flutter_rust_bridge::frb;
use std::collections::VecDeque;
use std::sync::{LazyLock, Mutex};

static EVENT_QUEUE: LazyLock<Mutex<VecDeque<AppEvent>>> =
    LazyLock::new(|| Mutex::new(VecDeque::new()));

pub async fn start_event_listener() -> Result<String, String> {
    let core_guard = CORE.lock().unwrap();
    if let Some(core) = core_guard.clone() {
        drop(core_guard);

        tokio::spawn(async move {
            let mut event_receiver = core.lock().unwrap().get_event_bus().subscribe();
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

#[frb(sync)]
pub fn get_pending_events() -> Vec<AppEvent> {
    let mut queue = EVENT_QUEUE.lock().unwrap();
    let events: Vec<AppEvent> = queue.drain(..).collect();
    events
}

#[frb(sync)]
pub fn has_pending_events() -> bool {
    let queue = EVENT_QUEUE.lock().unwrap();
    !queue.is_empty()
}

#[frb(sync)]
pub fn clear_event_queue() -> String {
    let mut queue = EVENT_QUEUE.lock().unwrap();
    queue.clear();
    "Event queue cleared".to_string()
}

pub async fn emit_custom_event(event_type: String, data: String) -> Result<String, String> {
    let core_guard = CORE.lock().unwrap();
    if let Some(core) = core_guard.clone() {
        drop(core_guard);

        let custom_event = AppEvent::Custom {
            event_type: event_type.clone(),
            data: data.clone(),
            timestamp: chrono::Utc::now().timestamp() as u64,
        };

        core.lock().unwrap().get_event_bus().emit(custom_event);
        Ok(format!("Custom event '{}' emitted: {}", event_type, data))
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb(sync)]
pub fn get_event_queue_size() -> u32 {
    let queue = EVENT_QUEUE.lock().unwrap();
    queue.len() as u32
}
