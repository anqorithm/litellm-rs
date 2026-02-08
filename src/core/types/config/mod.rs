//! Configuration types module
//!
//! Split from the original config.rs for better maintainability

pub mod defaults;
pub mod health;
pub mod middleware;
pub mod observability;
pub mod provider;
pub mod rate_limit;
pub mod retry;
pub mod routing;
pub mod server;

use self::{
    middleware::MiddlewareConfig, observability::ObservabilityConfig,
    provider::ProviderConfigEntry, routing::RoutingConfig, server::ServerConfig,
};
use serde::{Deserialize, Serialize};

/// Main LiteLLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteLLMConfig {
    /// Server configuration
    pub server: ServerConfig,
    /// Provider configurations
    pub providers: Vec<ProviderConfigEntry>,
    /// Routing configuration
    pub routing: RoutingConfig,
    /// Middleware configuration
    pub middleware: MiddlewareConfig,
    /// Observability configuration
    pub observability: ObservabilityConfig,
}

// Duration serialization module (shared across config types)
pub mod duration_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}
