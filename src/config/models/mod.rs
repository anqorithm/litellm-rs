//! Configuration data models
//!
//! This module defines all configuration structures used throughout the gateway.

#![allow(missing_docs)]

pub mod auth;
pub mod budget;
pub mod cache;
pub mod enterprise;
pub mod file_storage;
pub mod gateway;
pub mod monitoring;
pub mod provider;
pub mod rate_limit;
pub mod router;
pub mod server;
pub mod storage;

/// Default values for configuration
pub fn default_host() -> String {
    "0.0.0.0".to_string()
}

/// Default server port
pub fn default_port() -> u16 {
    8000
}

/// Default timeout in seconds
pub fn default_timeout() -> u64 {
    30
}

/// Default maximum body size in bytes
pub fn default_max_body_size() -> usize {
    10 * 1024 * 1024 // 10MB
}

/// Default maximum retry attempts
pub fn default_max_retries() -> u32 {
    3
}

/// Default provider weight
pub fn default_weight() -> f32 {
    1.0
}

pub fn default_rpm() -> u32 {
    1000
}

pub fn default_tpm() -> u32 {
    100_000
}

pub fn default_cache_ttl() -> u64 {
    3600 // 1 hour
}

pub fn default_cache_max_size() -> usize {
    1000
}

pub fn default_similarity_threshold() -> f64 {
    0.95
}

pub fn default_health_check_interval() -> u64 {
    30
}

pub fn default_failure_threshold() -> u32 {
    5
}

pub fn default_recovery_timeout() -> u64 {
    60
}

pub fn default_min_requests() -> u32 {
    10
}

pub fn default_base_delay() -> u64 {
    100
}

pub fn default_max_delay() -> u64 {
    5000
}

pub fn default_backoff_multiplier() -> f64 {
    2.0
}

pub fn default_max_connections() -> u32 {
    10
}

pub fn default_connection_timeout() -> u64 {
    5
}

pub fn default_redis_max_connections() -> u32 {
    20
}

pub fn default_jwt_expiration() -> u64 {
    86400 // 24 hours
}

pub fn default_api_key_header() -> String {
    "Authorization".to_string()
}

pub fn default_role() -> String {
    "user".to_string()
}

pub fn default_admin_roles() -> Vec<String> {
    vec!["admin".to_string(), "superuser".to_string()]
}

pub fn default_metrics_port() -> u16 {
    9090
}

pub fn default_metrics_path() -> String {
    "/metrics".to_string()
}

pub fn default_health_path() -> String {
    "/health".to_string()
}

pub fn default_service_name() -> String {
    "litellm-rs".to_string()
}
