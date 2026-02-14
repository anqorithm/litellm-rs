//! Legacy configuration types module.
//!
//! This module is kept for compatibility only.
//! Canonical configuration boundaries:
//! - Gateway/server runtime config: `crate::config::models::*`
//! - SDK/client config: `crate::sdk::config::*`

#[doc(hidden)]
pub mod defaults;
#[doc(hidden)]
pub mod health;
#[doc(hidden)]
pub mod middleware;
#[doc(hidden)]
pub mod observability;
#[doc(hidden)]
pub mod provider;
#[doc(hidden)]
pub mod rate_limit;
#[doc(hidden)]
pub mod retry;
#[doc(hidden)]
pub mod routing;

use self::{
    middleware::MiddlewareConfig, observability::ObservabilityConfig,
    provider::ProviderConfigEntry, routing::RoutingConfig,
};
use crate::config::models::server::ServerConfig;
use serde::{Deserialize, Serialize};

/// Main LiteLLM configuration
#[deprecated(
    note = "Use crate::config::models::gateway::GatewayConfig for server runtime config or crate::sdk::config::ClientConfig for SDK client config."
)]
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

/// Legacy compatibility alias to avoid semantic collision with gateway config types.
pub type LegacyProviderConfigEntry = provider::ProviderConfigEntry;

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
