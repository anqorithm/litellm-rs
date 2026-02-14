//! Tests for optimized Redis storage

#[cfg(test)]
use super::pool::OptimizedRedisPool;
use super::types::{PoolConfig, PoolStats};
use std::time::Duration;

#[test]
fn test_pool_config_default() {
    let config = PoolConfig::default();
    assert_eq!(config.max_connections, 20);
    assert_eq!(config.min_connections, 5);
    assert_eq!(config.connection_timeout, Duration::from_secs(5));
}

#[test]
fn test_pool_stats_default() {
    let stats = PoolStats::default();
    assert_eq!(stats.total_connections, 0);
    assert_eq!(stats.active_connections, 0);
    assert_eq!(stats.total_requests, 0);
}

#[test]
fn test_sanitize_url() {
    let url_with_auth = "redis://user:pass@localhost:6379";
    let sanitized = OptimizedRedisPool::sanitize_url(url_with_auth);
    assert_eq!(sanitized, "redis://[REDACTED]@localhost:6379");

    let url_without_auth = "redis://localhost:6379";
    let sanitized = OptimizedRedisPool::sanitize_url(url_without_auth);
    assert_eq!(sanitized, "redis://localhost:6379");
}
