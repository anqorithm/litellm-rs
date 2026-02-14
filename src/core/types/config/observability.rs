//! Observability configuration types
//!
//! This module re-exports from the canonical `crate::config::models::monitoring`.
//! Kept for backward compatibility with the legacy `LiteLLMConfig`.

pub use crate::config::models::monitoring::{
    HealthConfig, JaegerConfig, LogFormat, LogOutput, LoggingConfig, MetricsConfig,
    MonitoringConfig, TracingConfig,
};

/// Legacy alias — use [`MonitoringConfig`] directly.
pub type ObservabilityConfig = MonitoringConfig;
