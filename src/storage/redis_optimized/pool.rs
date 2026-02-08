//! Optimized Redis connection pool implementation
//!
//! Provides connection pooling, batch operations, and performance monitoring
//! for high-throughput Redis operations.

use super::connection::ConnectionPool;
use super::types::{PoolConfig, PoolStats};
use crate::config::RedisConfig;
use crate::utils::error::error::{GatewayError, Result};
use dashmap::DashMap;
use redis::{Client, RedisResult, aio::MultiplexedConnection};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Optimized Redis connection pool with advanced features
///
/// Provides connection pooling, batch operations, and performance monitoring
/// for high-throughput Redis operations.
pub struct OptimizedRedisPool {
    /// Connection pool manager
    pool: ConnectionPool,
    /// Redis configuration (stored for reconnection scenarios)
    #[allow(dead_code)]
    config: RedisConfig,
    /// Pool statistics for monitoring
    stats: Arc<RwLock<PoolStats>>,
    /// Response time tracking per operation type
    #[allow(dead_code)] // Reserved for detailed performance analytics
    response_times: Arc<DashMap<String, Vec<Duration>>>,
}

impl OptimizedRedisPool {
    /// Create a new optimized Redis pool
    ///
    /// # Arguments
    /// * `config` - Redis connection configuration
    /// * `pool_config` - Pool tuning parameters
    ///
    /// # Returns
    /// A new `OptimizedRedisPool` instance with initialized connections
    pub async fn new(config: &RedisConfig, pool_config: PoolConfig) -> Result<Self> {
        info!("Creating optimized Redis connection pool");
        debug!("Redis URL: {}", Self::sanitize_url(&config.url));

        let client = Client::open(config.url.as_str()).map_err(GatewayError::Redis)?;
        let pool = ConnectionPool::new(client, pool_config);
        let stats = Arc::new(RwLock::new(PoolStats::default()));
        let response_times = Arc::new(DashMap::new());

        let redis_pool = Self {
            pool,
            config: config.clone(),
            stats,
            response_times,
        };

        // Initialize minimum connections
        redis_pool.pool.initialize_connections().await?;

        // Start background tasks
        redis_pool.pool.start_health_checker();
        redis_pool.pool.start_connection_manager();

        info!("Optimized Redis connection pool created successfully");
        Ok(redis_pool)
    }

    /// Execute a Redis command with performance tracking
    ///
    /// Automatically handles connection acquisition, execution, and return to pool.
    /// Also updates pool statistics for monitoring.
    pub async fn execute_command<T, F, Fut>(&self, operation: F) -> Result<T>
    where
        F: FnOnce(MultiplexedConnection) -> Fut,
        Fut: std::future::Future<Output = RedisResult<T>>,
    {
        let start_time = Instant::now();
        let connection = self.pool.get_connection().await?;

        let result = operation(connection.clone()).await;
        let duration = start_time.elapsed();

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.total_requests += 1;

            match &result {
                Ok(_) => {
                    // Update average response time
                    let total_time =
                        stats.average_response_time_ms * (stats.total_requests - 1) as f64;
                    stats.average_response_time_ms =
                        (total_time + duration.as_millis() as f64) / stats.total_requests as f64;
                }
                Err(_) => {
                    stats.failed_requests += 1;
                }
            }
        }

        // Return connection to pool
        self.pool.return_connection(connection).await;

        result.map_err(GatewayError::Redis)
    }

    /// Batch set operations with pipeline for better performance
    ///
    /// Uses Redis pipeline for atomic batch operations.
    pub async fn batch_set(&self, pairs: &[(String, String)], ttl: Option<u64>) -> Result<()> {
        if pairs.is_empty() {
            return Ok(());
        }

        self.execute_command(|conn| async move {
            super::operations::batch_set_operation(conn, pairs, ttl).await
        })
        .await
    }

    /// Batch get operations using MGET
    pub async fn batch_get(&self, keys: &[String]) -> Result<Vec<Option<String>>> {
        if keys.is_empty() {
            return Ok(Vec::new());
        }

        self.execute_command(|conn| async move {
            super::operations::batch_get_operation(conn, keys).await
        })
        .await
    }

    /// Batch delete operations
    ///
    /// Returns the number of keys deleted.
    pub async fn batch_delete(&self, keys: &[String]) -> Result<u64> {
        if keys.is_empty() {
            return Ok(0);
        }

        self.execute_command(|conn| async move {
            super::operations::batch_delete_operation(conn, keys).await
        })
        .await
    }

    /// Get pool statistics for monitoring
    ///
    /// Returns current pool metrics including connection counts and performance data.
    pub async fn get_stats(&self) -> PoolStats {
        let connections = self.pool.connections.read().await;
        let mut stats = self.stats.read().await.clone();

        stats.total_connections = connections.len();
        stats.active_connections =
            self.pool.pool_config.max_connections - self.pool.available_permits();
        stats.idle_connections = connections
            .iter()
            .filter(|c| c.is_idle(self.pool.pool_config.max_idle_time))
            .count();

        stats
    }

    /// Sanitize URL for logging (redacts credentials)
    pub(super) fn sanitize_url(url: &str) -> String {
        if let Some(at_pos) = url.find('@') {
            if let Some(scheme_end) = url.find("://") {
                format!("{}://[REDACTED]@{}", &url[..scheme_end], &url[at_pos + 1..])
            } else {
                "[REDACTED]".to_string()
            }
        } else {
            url.to_string()
        }
    }
}
