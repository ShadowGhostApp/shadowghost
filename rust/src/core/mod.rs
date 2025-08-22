// core/mod.rs - Улучшенная структура ядра
pub mod config;
pub mod engine;
pub mod health;
pub mod lifecycle;
pub mod metrics;
pub mod peer;

pub use config::*;
pub use engine::*;
pub use health::*;
pub use lifecycle::*;
pub use metrics::*;
pub use peer::*;

use crate::data::DataManager;
use crate::events::EventBus;
use crate::network::NetworkManager;
use crate::security::SecurityManager;
use std::error::Error;
use std::fmt;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug)]
pub enum CoreError {
    InvalidState(String),
    Network(String),
    Storage(String),
    Contact(String),
    Config(String),
    Crypto(String),
    Initialization(String),
    Security(String),
    Lifecycle(String),
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreError::InvalidState(msg) => write!(f, "Invalid state: {}", msg),
            CoreError::Network(msg) => write!(f, "Network error: {}", msg),
            CoreError::Storage(msg) => write!(f, "Storage error: {}", msg),
            CoreError::Contact(msg) => write!(f, "Contact error: {}", msg),
            CoreError::Config(msg) => write!(f, "Config error: {}", msg),
            CoreError::Crypto(msg) => write!(f, "Crypto error: {}", msg),
            CoreError::Initialization(msg) => write!(f, "Initialization error: {}", msg),
            CoreError::Security(msg) => write!(f, "Security error: {}", msg),
            CoreError::Lifecycle(msg) => write!(f, "Lifecycle error: {}", msg),
        }
    }
}

impl Error for CoreError {}

/// Состояния жизненного цикла ядра
#[derive(Debug, Clone, PartialEq)]
pub enum CoreState {
    Uninitialized,
    Initializing,
    Ready,
    Running,
    Stopping,
    Stopped,
    Error(String),
}

/// Главное ядро ShadowGhost с улучшенной архитектурой
pub struct ShadowGhostCore {
    // Конфигурация
    config_manager: ConfigManager,

    // Основные менеджеры
    data_manager: Arc<RwLock<DataManager>>,
    network_manager: Arc<RwLock<NetworkManager>>,
    security_manager: Arc<RwLock<SecurityManager>>,

    // Системы мониторинга
    health_monitor: Arc<RwLock<HealthMonitor>>,
    metrics_collector: Arc<RwLock<MetricsCollector>>,
    lifecycle_manager: Arc<RwLock<LifecycleManager>>,

    // События и состояние
    event_bus: EventBus,
    state: CoreState,
    user_name: Option<String>,
    peer: Peer,
}

impl ShadowGhostCore {
    pub fn new() -> Result<Self, CoreError> {
        let config_manager =
            ConfigManager::new("./config.toml").map_err(|e| CoreError::Config(e.to_string()))?;

        let event_bus = EventBus::new();
        let peer = Peer::new("default_user".to_string(), "127.0.0.1:8080".to_string());

        // Создаем менеджеры
        let config = config_manager.get_config().clone();

        Ok(Self {
            config_manager,
            data_manager: Arc::new(RwLock::new(DataManager::new(
                config.clone(),
                event_bus.clone(),
            )?)),
            network_manager: Arc::new(RwLock::new(NetworkManager::new(
                peer.clone(),
                event_bus.clone(),
            )?)),
            security_manager: Arc::new(RwLock::new(SecurityManager::new(
                config.clone(),
                event_bus.clone(),
            )?)),
            health_monitor: Arc::new(RwLock::new(HealthMonitor::new(config.clone()))),
            metrics_collector: Arc::new(RwLock::new(MetricsCollector::new(config.clone()))),
            lifecycle_manager: Arc::new(RwLock::new(LifecycleManager::new(config.clone()))),
            event_bus,
            state: CoreState::Uninitialized,
            user_name: None,
            peer,
        })
    }

    pub fn new_for_test(test_id: &str) -> Result<Self, CoreError> {
        let temp_dir = std::env::temp_dir().join("shadowghost_test").join(test_id);
        std::fs::create_dir_all(&temp_dir).map_err(|e| CoreError::Config(e.to_string()))?;

        let config_path = temp_dir.join("config.toml");
        let config_manager =
            ConfigManager::new(config_path).map_err(|e| CoreError::Config(e.to_string()))?;

        let event_bus = EventBus::new();
        let peer = Peer::new("test_user".to_string(), "127.0.0.1:8080".to_string());

        let config = config_manager.get_config().clone();

        Ok(Self {
            config_manager,
            data_manager: Arc::new(RwLock::new(DataManager::new(
                config.clone(),
                event_bus.clone(),
            )?)),
            network_manager: Arc::new(RwLock::new(NetworkManager::new(
                peer.clone(),
                event_bus.clone(),
            )?)),
            security_manager: Arc::new(RwLock::new(SecurityManager::new(
                config.clone(),
                event_bus.clone(),
            )?)),
            health_monitor: Arc::new(RwLock::new(HealthMonitor::new(config.clone()))),
            metrics_collector: Arc::new(RwLock::new(MetricsCollector::new(config.clone()))),
            lifecycle_manager: Arc::new(RwLock::new(LifecycleManager::new(config.clone()))),
            event_bus,
            state: CoreState::Uninitialized,
            user_name: None,
            peer,
        })
    }

    /// Инициализация ядра
    pub async fn initialize(&mut self, user_name: Option<String>) -> Result<(), CoreError> {
        if !matches!(self.state, CoreState::Uninitialized) {
            return Err(CoreError::InvalidState(
                "Core already initialized".to_string(),
            ));
        }

        self.state = CoreState::Initializing;

        // Настройка пользователя
        if let Some(name) = user_name {
            self.config_manager
                .set_user_name(name.clone())
                .map_err(|e| CoreError::Config(e.to_string()))?;
            self.user_name = Some(name);
            self.peer
                .update_name(self.user_name.as_ref().unwrap().clone());
        } else {
            self.user_name = Some(self.config_manager.get_user_name().to_string());
            self.peer
                .update_name(self.user_name.as_ref().unwrap().clone());
        }

        // Инициализируем менеджеры в правильном порядке
        self.security_manager
            .write()
            .await
            .initialize()
            .await
            .map_err(|e| CoreError::Security(e.to_string()))?;

        self.data_manager
            .write()
            .await
            .initialize()
            .await
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        self.network_manager
            .write()
            .await
            .start()
            .map_err(|e| CoreError::Network(e.to_string()))?;

        // Запускаем системы мониторинга
        self.health_monitor
            .write()
            .await
            .start()
            .await
            .map_err(|e| CoreError::Initialization(e.to_string()))?;

        self.metrics_collector
            .write()
            .await
            .start()
            .await
            .map_err(|e| CoreError::Initialization(e.to_string()))?;

        self.lifecycle_manager
            .write()
            .await
            .initialize()
            .await
            .map_err(|e| CoreError::Lifecycle(e.to_string()))?;

        self.state = CoreState::Ready;
        Ok(())
    }

    /// Получение состояния ядра
    pub fn get_state(&self) -> CoreState {
        self.state.clone()
    }

    /// Проверка инициализации
    pub fn is_initialized(&self) -> bool {
        matches!(self.state, CoreState::Ready | CoreState::Running)
    }

    /// Запуск сервера
    pub async fn start_server(&mut self) -> Result<(), CoreError> {
        if !self.is_initialized() {
            return Err(CoreError::InvalidState("Core not initialized".to_string()));
        }

        self.state = CoreState::Running;

        self.network_manager
            .write()
            .await
            .start_server()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        // Уведомляем о запуске
        self.event_bus
            .emit_network(crate::events::NetworkEvent::ServerStarted {
                port: self.config_manager.get_network_port(),
            });

        Ok(())
    }

    /// Остановка сервера
    pub async fn stop_server(&mut self) -> Result<(), CoreError> {
        self.state = CoreState::Stopping;

        self.network_manager
            .write()
            .await
            .shutdown()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        // Уведомляем об остановке
        self.event_bus
            .emit_network(crate::events::NetworkEvent::ServerStopped);

        self.state = CoreState::Ready;
        Ok(())
    }

    /// Перезапуск сервера
    pub async fn restart_server(&mut self) -> Result<(), CoreError> {
        self.stop_server().await?;
        tokio::time::sleep(std::time::Duration::from_millis(100)).await; // Небольшая пауза
        self.start_server().await?;
        Ok(())
    }

    /// Проверка запуска сервера
    pub fn is_server_started(&self) -> bool {
        matches!(self.state, CoreState::Running)
    }

    /// Получение статуса сервера
    pub async fn get_server_status(&self) -> String {
        match self.state {
            CoreState::Running => "🟢 Running".to_string(),
            CoreState::Ready => "🟡 Ready".to_string(),
            CoreState::Initializing => "🟠 Initializing".to_string(),
            CoreState::Stopping => "🟠 Stopping".to_string(),
            CoreState::Stopped => "🔴 Stopped".to_string(),
            CoreState::Uninitialized => "⚫ Uninitialized".to_string(),
            CoreState::Error(ref msg) => format!("❌ Error: {}", msg),
        }
    }

    /// Получение информации о пире
    pub async fn get_peer_info(&self) -> Option<String> {
        if let Some(ref name) = self.user_name {
            Some(format!("{} ({})", name, self.peer.get_full_address()))
        } else {
            None
        }
    }

    /// Генерация SG ссылки
    pub async fn generate_sg_link(&self) -> Result<String, CoreError> {
        if !self.is_initialized() {
            return Err(CoreError::InvalidState("Core not initialized".to_string()));
        }

        // Получаем публичный ключ
        let public_key = self
            .security_manager
            .read()
            .await
            .crypto
            .read()
            .await
            .get_public_key();

        let peer_data = crate::network::PeerData {
            id: self.peer.id.clone(),
            name: self.peer.name.clone(),
            address: self.peer.get_full_address(),
            public_key: public_key.key_data,
            connected_at: chrono::Utc::now(),
            last_seen: chrono::Utc::now(),
            bytes_sent: 0,
            bytes_received: 0,
        };

        let json_data = serde_json::to_string(&peer_data)
            .map_err(|e| CoreError::Contact(format!("Failed to serialize peer data: {}", e)))?;

        use base64::{engine::general_purpose, Engine as _};
        let encoded = general_purpose::STANDARD.encode(json_data);
        Ok(format!("sg://{}", encoded))
    }

    /// Добавление контакта по SG ссылке
    pub async fn add_contact_by_sg_link(&self, sg_link: &str) -> Result<(), CoreError> {
        if !self.is_initialized() {
            return Err(CoreError::InvalidState("Core not initialized".to_string()));
        }

        // Парсим ссылку и создаем контакт
        let contact = self.parse_sg_link(sg_link).await?;

        // Добавляем контакт
        self.data_manager
            .write()
            .await
            .contacts
            .write()
            .await
            .add_contact(contact)
            .map_err(|e| CoreError::Contact(e.to_string()))?;

        Ok(())
    }

    /// Получение контактов
    pub async fn get_contacts(&self) -> Result<Vec<crate::network::Contact>, CoreError> {
        if !self.is_initialized() {
            return Err(CoreError::InvalidState("Core not initialized".to_string()));
        }

        self.data_manager
            .read()
            .await
            .contacts
            .read()
            .await
            .get_contacts()
            .await
            .map_err(|e| CoreError::Contact(e.to_string()))
    }

    /// Получение количества контактов
    pub async fn get_contact_count(&self) -> usize {
        if let Ok(contacts) = self.get_contacts().await {
            contacts.len()
        } else {
            0
        }
    }

    /// Отправка сообщения
    pub async fn send_message(&self, contact_name: &str, content: &str) -> Result<(), CoreError> {
        if !self.is_initialized() {
            return Err(CoreError::InvalidState("Core not initialized".to_string()));
        }

        // Валидация контента
        if content.trim().is_empty() {
            return Err(CoreError::InvalidState(
                "Message content cannot be empty".to_string(),
            ));
        }

        if content.len() > 10000 {
            return Err(CoreError::InvalidState(
                "Message content too long".to_string(),
            ));
        }

        // Находим контакт
        let contacts = self.get_contacts().await?;
        let contact = contacts
            .iter()
            .find(|c| c.name == contact_name)
            .ok_or_else(|| CoreError::Contact(format!("Contact '{}' not found", contact_name)))?;

        // Отправляем сообщение через сетевой менеджер
        let message_id = self
            .network_manager
            .read()
            .await
            .send_chat_message(contact, content)
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        // Сохраняем сообщение
        let message = crate::network::ChatMessage {
            id: message_id,
            from: self
                .user_name
                .as_ref()
                .unwrap_or(&"user".to_string())
                .clone(),
            to: contact_name.to_string(),
            content: content.to_string(),
            msg_type: crate::network::ChatMessageType::Text,
            timestamp: chrono::Utc::now().timestamp() as u64,
            delivery_status: crate::network::DeliveryStatus::Sent,
        };

        self.data_manager
            .write()
            .await
            .storage
            .write()
            .await
            .save_message(contact_name, &message)
            .await
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        Ok(())
    }

    /// Получение сообщений чата
    pub async fn get_chat_messages(
        &self,
        contact_name: &str,
    ) -> Result<Vec<crate::network::ChatMessage>, CoreError> {
        if !self.is_initialized() {
            return Err(CoreError::InvalidState("Core not initialized".to_string()));
        }

        self.data_manager
            .read()
            .await
            .storage
            .read()
            .await
            .get_messages(contact_name)
            .await
            .map_err(|e| CoreError::Storage(e.to_string()))
    }

    /// Получение статистики сети
    pub async fn get_network_stats(&self) -> Result<crate::network::NetworkStats, CoreError> {
        self.network_manager
            .read()
            .await
            .get_network_stats()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))
    }

    /// Обновление имени пользователя
    pub async fn update_user_name(&mut self, new_name: String) -> Result<(), CoreError> {
        self.config_manager
            .set_user_name(new_name.clone())
            .map_err(|e| CoreError::Config(e.to_string()))?;

        self.user_name = Some(new_name.clone());
        self.peer.update_name(new_name);

        Ok(())
    }

    /// Получение информации о соединении
    pub async fn get_connection_info(&self) -> Result<String, CoreError> {
        Ok(format!(
            "Address: {}\nPort: {}\nState: {:?}",
            self.peer.address, self.peer.port, self.state
        ))
    }

    /// Обновление внешнего адреса
    pub async fn update_external_address(&self) -> Result<(), CoreError> {
        // В реальной реализации здесь было бы обновление внешнего IP
        Ok(())
    }

    /// Корректное завершение работы
    pub async fn shutdown(&mut self) -> Result<(), CoreError> {
        if matches!(self.state, CoreState::Stopped | CoreState::Uninitialized) {
            return Ok(());
        }

        self.state = CoreState::Stopping;

        // Останавливаем все компоненты в обратном порядке
        if let Err(e) = self.network_manager.write().await.shutdown().await {
            eprintln!("Warning: Network shutdown error: {}", e);
        }

        if let Err(e) = self.data_manager.write().await.shutdown().await {
            eprintln!("Warning: Data shutdown error: {}", e);
        }

        if let Err(e) = self.health_monitor.write().await.stop().await {
            eprintln!("Warning: Health monitor shutdown error: {}", e);
        }

        if let Err(e) = self.metrics_collector.write().await.stop().await {
            eprintln!("Warning: Metrics collector shutdown error: {}", e);
        }

        self.state = CoreState::Stopped;
        Ok(())
    }

    /// Получение шины событий
    pub fn get_event_bus(&self) -> EventBus {
        self.event_bus.clone()
    }

    /// Получение метрик системы
    pub async fn get_system_metrics(&self) -> SystemMetrics {
        if let Ok(metrics) = self
            .metrics_collector
            .read()
            .await
            .get_current_metrics()
            .await
        {
            metrics
        } else {
            SystemMetrics::default()
        }
    }

    /// Получение состояния здоровья системы
    pub async fn get_health_status(&self) -> HealthStatus {
        if let Ok(status) = self.health_monitor.read().await.get_health_status().await {
            status
        } else {
            HealthStatus::unhealthy("Failed to get health status".to_string())
        }
    }

    // Приватные методы для внутренней логики
    async fn parse_sg_link(&self, sg_link: &str) -> Result<crate::network::Contact, CoreError> {
        if !sg_link.starts_with("sg://") {
            return Err(CoreError::Contact("Invalid SG link format".to_string()));
        }

        let link_data = &sg_link[5..];
        use base64::{engine::general_purpose, Engine as _};

        let decoded_data = general_purpose::STANDARD
            .decode(link_data)
            .map_err(|e| CoreError::Contact(format!("Failed to decode SG link: {}", e)))?;

        let data_str = String::from_utf8(decoded_data)
            .map_err(|_| CoreError::Contact("Invalid UTF-8 in SG link".to_string()))?;

        let peer_data: crate::network::PeerData = serde_json::from_str(&data_str)
            .map_err(|_| CoreError::Contact("Invalid JSON in SG link".to_string()))?;

        if peer_data.name == self.peer.name {
            return Err(CoreError::Contact(
                "Cannot add yourself as contact".to_string(),
            ));
        }

        Ok(crate::network::Contact {
            id: peer_data.id,
            name: peer_data.name,
            address: peer_data.address,
            status: crate::network::ContactStatus::Offline,
            trust_level: crate::network::TrustLevel::Pending,
            last_seen: Some(peer_data.last_seen),
        })
    }

    // Заглушки для совместимости с текущим API
    pub async fn add_contact_manual(
        &self,
        contact: crate::network::Contact,
    ) -> Result<(), CoreError> {
        self.data_manager
            .write()
            .await
            .contacts
            .write()
            .await
            .add_contact(contact)
            .map_err(|e| CoreError::Contact(e.to_string()))
    }

    pub fn get_contacts_sync(&self) -> Result<Vec<crate::network::Contact>, CoreError> {
        // Заглушка для синхронного API
        Ok(vec![])
    }

    pub async fn remove_contact_by_id(&self, contact_id: &str) -> Result<(), CoreError> {
        self.data_manager
            .write()
            .await
            .contacts
            .write()
            .await
            .remove_contact(contact_id)
            .map_err(|e| CoreError::Contact(e.to_string()))
    }

    pub async fn update_contact_trust_level(
        &self,
        contact_id: &str,
        trust_level: crate::network::TrustLevel,
    ) -> Result<(), CoreError> {
        self.data_manager
            .write()
            .await
            .contacts
            .write()
            .await
            .set_trust_level(contact_id, trust_level)
            .map_err(|e| CoreError::Contact(e.to_string()))
    }

    pub fn get_contact_by_id(&self, _contact_id: &str) -> Option<crate::network::Contact> {
        // Заглушка
        None
    }

    pub async fn send_chat_message(&self, to: &str, content: &str) -> Result<(), CoreError> {
        self.send_message(to, content).await
    }

    pub fn get_unread_count(&self, _contact_id: &str) -> Result<usize, CoreError> {
        // Заглушка
        Ok(0)
    }

    pub async fn check_contact_online(&self, _contact_name: &str) -> bool {
        // Заглушка
        false
    }

    pub async fn get_unread_message_count(&self, _contact_name: &str) -> Result<usize, CoreError> {=
        Ok(0)
    }
}
