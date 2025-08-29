use crate::core::types::Config;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LifecycleState {
    Uninitialized,
    Initializing,
    Starting,
    Running,
    Stopping,
    Stopped,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleEvent {
    pub state: LifecycleState,
    pub timestamp: u64,
    pub message: String,
    pub component: Option<String>,
}

pub struct LifecycleManager {
    config: Config,
    current_state: LifecycleState,
    event_history: Vec<LifecycleEvent>,
    start_time: Option<SystemTime>,
}

impl LifecycleManager {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            current_state: LifecycleState::Uninitialized,
            event_history: Vec::new(),
            start_time: None,
        }
    }

    pub async fn initialize(&mut self) -> Result<(), String> {
        self.transition_to(
            LifecycleState::Initializing,
            "System initialization started".to_string(),
        )
        .await?;

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        self.transition_to(
            LifecycleState::Running,
            "System initialized successfully".to_string(),
        )
        .await?;
        self.start_time = Some(SystemTime::now());

        Ok(())
    }

    pub async fn start(&mut self) -> Result<(), String> {
        if self.current_state != LifecycleState::Running {
            return Err("System must be initialized before starting".to_string());
        }

        self.transition_to(
            LifecycleState::Starting,
            "System startup initiated".to_string(),
        )
        .await?;

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        self.transition_to(
            LifecycleState::Running,
            "System fully operational".to_string(),
        )
        .await?;

        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), String> {
        self.transition_to(
            LifecycleState::Stopping,
            "System shutdown initiated".to_string(),
        )
        .await?;

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        self.transition_to(
            LifecycleState::Stopped,
            "System stopped successfully".to_string(),
        )
        .await?;

        Ok(())
    }

    pub async fn emergency_stop(&mut self, reason: String) -> Result<(), String> {
        self.transition_to(LifecycleState::Error, format!("Emergency stop: {}", reason))
            .await?;
        Ok(())
    }

    async fn transition_to(
        &mut self,
        new_state: LifecycleState,
        message: String,
    ) -> Result<(), String> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let event = LifecycleEvent {
            state: new_state.clone(),
            timestamp,
            message: message.clone(),
            component: None,
        };

        self.event_history.push(event);

        if self.event_history.len() > 50 {
            self.event_history.remove(0);
        }

        self.current_state = new_state;

        println!("Lifecycle transition: {}", message);

        Ok(())
    }

    pub fn get_current_state(&self) -> LifecycleState {
        self.current_state.clone()
    }

    pub fn get_event_history(&self) -> &Vec<LifecycleEvent> {
        &self.event_history
    }

    pub fn get_uptime(&self) -> Option<std::time::Duration> {
        self.start_time.and_then(|start| start.elapsed().ok())
    }

    pub fn is_running(&self) -> bool {
        self.current_state == LifecycleState::Running
    }

    pub fn is_stopped(&self) -> bool {
        matches!(
            self.current_state,
            LifecycleState::Stopped | LifecycleState::Error
        )
    }

    pub fn can_start(&self) -> bool {
        matches!(
            self.current_state,
            LifecycleState::Running | LifecycleState::Stopped
        )
    }

    pub fn can_stop(&self) -> bool {
        self.current_state == LifecycleState::Running
    }

    pub async fn record_component_event(
        &mut self,
        component: String,
        message: String,
    ) -> Result<(), String> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let event = LifecycleEvent {
            state: self.current_state.clone(),
            timestamp,
            message,
            component: Some(component),
        };

        self.event_history.push(event);

        if self.event_history.len() > 50 {
            self.event_history.remove(0);
        }

        Ok(())
    }

    pub fn get_state_duration(&self) -> std::time::Duration {
        if let Some(last_event) = self.event_history.last() {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            std::time::Duration::from_secs(now - last_event.timestamp)
        } else {
            std::time::Duration::from_secs(0)
        }
    }
}
