//! Type definitions for optimized Redis storage
//!
//! Contains configuration types, statistics, and connection metadata.

use std::time::{Duration, Instant};

/// Connection pool statistics for monitoring Redis performance
#[derive(Debug, Clone, Default)]
pub struct PoolStats {
    /// Total number of connections in the pool
    pub total_connections: usize,
    /// Number of actively used connections
    pub active_connections: usize,
    /// Number of idle connections waiting to be used
    pub idle_connections: usize,
    /// Total number of requests processed
    pub total_requests: u64,
    /// Number of failed requests
    pub failed_requests: u64,
    /// Average response time in milliseconds
    pub average_response_time_ms: f64,
}

/// Connection pool configuration for tuning Redis performance
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum number of connections in the pool
    pub max_connections: usize,
    /// Minimum number of connections to maintain
    pub min_connections: usize,
    /// Connection timeout in seconds
    pub connection_timeout: Duration,
    /// Maximum idle time before connection is closed
    pub max_idle_time: Duration,
    /// Health check interval for background monitoring
    pub health_check_interval: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 20,
            min_connections: 5,
            connection_timeout: Duration::from_secs(5),
            max_idle_time: Duration::from_secs(300), // 5 minutes
            health_check_interval: Duration::from_secs(30),
        }
    }
}

/// Connection wrapper with metadata for pool management
#[derive(Debug)]
pub(super) struct PooledConnection {
    /// The actual Redis connection
    pub(super) connection: redis::aio::MultiplexedConnection,
    /// When this connection was created (for connection age tracking)
    pub(super) _created_at: Instant,
    /// When this connection was last used
    pub(super) last_used: Instant,
    /// Number of requests processed by this connection
    pub(super) request_count: u64,
    /// Whether the connection is healthy
    pub(super) is_healthy: bool,
}

impl PooledConnection {
    /// Create a new pooled connection wrapper
    pub(super) fn new(connection: redis::aio::MultiplexedConnection) -> Self {
        let now = Instant::now();
        Self {
            connection,
            _created_at: now,
            last_used: now,
            request_count: 0,
            is_healthy: true,
        }
    }

    /// Mark the connection as recently used
    pub(super) fn mark_used(&mut self) {
        self.last_used = Instant::now();
        self.request_count += 1;
    }

    /// Check if connection has been idle longer than max_idle_time
    pub(super) fn is_idle(&self, max_idle_time: Duration) -> bool {
        self.last_used.elapsed() > max_idle_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== PoolStats Tests ====================

    #[test]
    fn test_pool_stats_default() {
        let stats = PoolStats::default();
        assert_eq!(stats.total_connections, 0);
        assert_eq!(stats.active_connections, 0);
        assert_eq!(stats.idle_connections, 0);
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.failed_requests, 0);
        assert_eq!(stats.average_response_time_ms, 0.0);
    }

    #[test]
    fn test_pool_stats_structure() {
        let stats = PoolStats {
            total_connections: 20,
            active_connections: 5,
            idle_connections: 15,
            total_requests: 1000,
            failed_requests: 10,
            average_response_time_ms: 5.5,
        };

        assert_eq!(stats.total_connections, 20);
        assert_eq!(stats.active_connections, 5);
        assert_eq!(stats.idle_connections, 15);
        assert_eq!(stats.total_requests, 1000);
        assert_eq!(stats.failed_requests, 10);
        assert!((stats.average_response_time_ms - 5.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_pool_stats_clone() {
        let stats = PoolStats {
            total_connections: 10,
            active_connections: 3,
            idle_connections: 7,
            total_requests: 500,
            failed_requests: 5,
            average_response_time_ms: 2.5,
        };

        let cloned = stats.clone();
        assert_eq!(stats.total_connections, cloned.total_connections);
        assert_eq!(stats.active_connections, cloned.active_connections);
        assert_eq!(stats.total_requests, cloned.total_requests);
    }

    // ==================== PoolConfig Tests ====================

    #[test]
    fn test_pool_config_default() {
        let config = PoolConfig::default();
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.min_connections, 5);
        assert_eq!(config.connection_timeout, Duration::from_secs(5));
        assert_eq!(config.max_idle_time, Duration::from_secs(300));
        assert_eq!(config.health_check_interval, Duration::from_secs(30));
    }

    #[test]
    fn test_pool_config_custom() {
        let config = PoolConfig {
            max_connections: 50,
            min_connections: 10,
            connection_timeout: Duration::from_secs(10),
            max_idle_time: Duration::from_secs(600),
            health_check_interval: Duration::from_secs(60),
        };

        assert_eq!(config.max_connections, 50);
        assert_eq!(config.min_connections, 10);
        assert_eq!(config.connection_timeout, Duration::from_secs(10));
        assert_eq!(config.max_idle_time, Duration::from_secs(600));
        assert_eq!(config.health_check_interval, Duration::from_secs(60));
    }

    #[test]
    fn test_pool_config_clone() {
        let config = PoolConfig::default();
        let cloned = config.clone();

        assert_eq!(config.max_connections, cloned.max_connections);
        assert_eq!(config.min_connections, cloned.min_connections);
        assert_eq!(config.connection_timeout, cloned.connection_timeout);
    }

    #[test]
    fn test_pool_config_zero_values() {
        let config = PoolConfig {
            max_connections: 0,
            min_connections: 0,
            connection_timeout: Duration::ZERO,
            max_idle_time: Duration::ZERO,
            health_check_interval: Duration::ZERO,
        };

        assert_eq!(config.max_connections, 0);
        assert_eq!(config.min_connections, 0);
        assert_eq!(config.connection_timeout, Duration::ZERO);
    }

    #[test]
    fn test_pool_config_large_values() {
        let config = PoolConfig {
            max_connections: 1000,
            min_connections: 100,
            connection_timeout: Duration::from_secs(3600),
            max_idle_time: Duration::from_secs(86400),
            health_check_interval: Duration::from_secs(300),
        };

        assert_eq!(config.max_connections, 1000);
        assert_eq!(config.max_idle_time, Duration::from_secs(86400));
    }

    // ==================== Integration-like Tests ====================

    #[test]
    fn test_pool_stats_calculate_utilization() {
        let stats = PoolStats {
            total_connections: 20,
            active_connections: 15,
            idle_connections: 5,
            total_requests: 1000,
            failed_requests: 50,
            average_response_time_ms: 10.0,
        };

        // Calculate utilization rate
        let utilization = if stats.total_connections > 0 {
            stats.active_connections as f64 / stats.total_connections as f64
        } else {
            0.0
        };
        assert!((utilization - 0.75).abs() < f64::EPSILON);

        // Calculate failure rate
        let failure_rate = if stats.total_requests > 0 {
            stats.failed_requests as f64 / stats.total_requests as f64
        } else {
            0.0
        };
        assert!((failure_rate - 0.05).abs() < f64::EPSILON);
    }

    #[test]
    fn test_pool_stats_zero_total_connections() {
        let stats = PoolStats::default();

        let utilization = if stats.total_connections > 0 {
            stats.active_connections as f64 / stats.total_connections as f64
        } else {
            0.0
        };
        assert_eq!(utilization, 0.0);
    }

    #[test]
    fn test_pool_config_reasonable_defaults() {
        let config = PoolConfig::default();

        // Min should be less than or equal to max
        assert!(config.min_connections <= config.max_connections);

        // Connection timeout should be reasonable (1 second to 1 minute)
        assert!(config.connection_timeout >= Duration::from_secs(1));
        assert!(config.connection_timeout <= Duration::from_secs(60));

        // Idle time should be reasonable
        assert!(config.max_idle_time >= Duration::from_secs(60));
    }
}
