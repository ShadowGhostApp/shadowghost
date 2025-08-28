pub mod config;
pub mod engine;
pub mod flutter_api;
pub mod health;
pub mod lifecycle;
pub mod metrics;
pub mod peer;
pub mod profile;
pub mod types;

pub use config::*;
pub use engine::*;
pub use flutter_api::*;
pub use health::*;
pub use lifecycle::*;
pub use metrics::*;
pub use peer::*;
pub use profile::*;
pub use types::*;

pub trait ShadowGhostCore {
    fn is_initialized(&self) -> bool;
    fn get_event_bus(&self) -> &crate::events::EventBus;
}

impl ShadowGhostCore for Engine {
    fn is_initialized(&self) -> bool {
        true // Engine is initialized when created
    }

    fn get_event_bus(&self) -> &crate::events::EventBus {
        &self.event_bus
    }
}
