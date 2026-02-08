//! Redis atomic operations and utilities
//!
//! This module provides atomic increment/decrement operations and utility functions.

use super::pool::RedisPool;
use crate::utils::error::error::{GatewayError, Result};
use redis::AsyncCommands;

impl RedisPool {
    /// Increment key value by delta
    pub async fn increment(&self, key: &str, delta: i64) -> Result<i64> {
        if self.noop_mode {
            return Ok(delta);
        }

        let mut conn = self.get_connection().await?;
        if let Some(ref mut c) = conn.conn {
            let new_value: i64 = c.incr(key, delta).await.map_err(GatewayError::Redis)?;
            Ok(new_value)
        } else {
            Ok(delta)
        }
    }

    /// Decrement a key by a delta value
    pub async fn decrement(&self, key: &str, delta: i64) -> Result<i64> {
        if self.noop_mode {
            return Ok(-delta);
        }

        let mut conn = self.get_connection().await?;
        if let Some(ref mut c) = conn.conn {
            let new_value: i64 = c.decr(key, delta).await.map_err(GatewayError::Redis)?;
            Ok(new_value)
        } else {
            Ok(-delta)
        }
    }

    /// Get Redis info
    pub async fn info(&self) -> Result<String> {
        if self.noop_mode {
            return Ok("Redis unavailable (no-op mode)".to_string());
        }

        let mut conn = self.get_connection().await?;
        if let Some(ref mut c) = conn.conn {
            let info: String = redis::cmd("INFO")
                .query_async(c)
                .await
                .map_err(GatewayError::Redis)?;
            Ok(info)
        } else {
            Ok("Redis unavailable".to_string())
        }
    }

    /// Flush database (use with caution)
    pub async fn flush_db(&self) -> Result<()> {
        if self.noop_mode {
            return Ok(());
        }

        let mut conn = self.get_connection().await?;
        if let Some(ref mut c) = conn.conn {
            let _: () = redis::cmd("FLUSHDB")
                .query_async(c)
                .await
                .map_err(GatewayError::Redis)?;
        }
        Ok(())
    }
}
