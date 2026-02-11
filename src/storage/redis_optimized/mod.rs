//! Optimized Redis storage implementation with connection pooling and batch operations
//!
//! This module provides enhanced Redis connectivity with improved performance
//! through connection pooling, batch operations, and intelligent caching.
//! It is experimental and only available with feature flag `redis-optimized`.
//!
//! # Usage
//!
//! ```rust,no_run
//! # use litellm_rs::storage::redis_optimized::pool::OptimizedRedisPool;
//! # use litellm_rs::storage::redis_optimized::types::PoolConfig;
//! # use litellm_rs::config::RedisConfig;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = RedisConfig { url: "redis://localhost:6379".to_string(), ..Default::default() };
//! let pool = OptimizedRedisPool::new(&config, PoolConfig::default()).await?;
//!
//! // Batch operations
//! pool.batch_set(&[("key1".into(), "value1".into())], Some(3600)).await?;
//! let values = pool.batch_get(&["key1".into()]).await?;
//!
//! // Get pool statistics
//! let stats = pool.get_stats().await;
//! println!("Active connections: {}", stats.active_connections);
//! # Ok(())
//! # }
//! ```

pub mod connection;
pub mod operations;
pub mod pool;
pub mod types;

#[cfg(test)]
mod tests;
