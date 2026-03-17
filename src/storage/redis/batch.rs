//! Batch Redis operations
//!
//! This module provides batch operations for efficient multi-key operations.

use super::pool::RedisPool;
use crate::utils::error::gateway_error::{GatewayError, Result};
use redis::AsyncCommands;

impl RedisPool {
    /// Get multiple keys at once
    pub async fn mget(&self, keys: &[String]) -> Result<Vec<Option<String>>> {
        if self.noop_mode {
            return Ok(vec![None; keys.len()]);
        }

        let mut conn = self.get_connection().await?;
        if let Some(ref mut c) = conn.conn {
            let values: Vec<Option<String>> = c.mget(keys).await.map_err(GatewayError::from)?;
            Ok(values)
        } else {
            Ok(vec![None; keys.len()])
        }
    }

    /// Set multiple key-value pairs with optional TTL
    pub async fn mset(&self, pairs: &[(String, String)], ttl: Option<u64>) -> Result<()> {
        if self.noop_mode || pairs.is_empty() {
            return Ok(());
        }

        let mut conn = self.get_connection().await?;
        if let Some(ref mut c) = conn.conn {
            // Use atomic pipeline for better performance and consistency
            let mut pipe = redis::pipe();
            pipe.atomic();

            for (key, value) in pairs {
                if let Some(ttl_seconds) = ttl {
                    pipe.set_ex(key, value, ttl_seconds);
                } else {
                    pipe.set(key, value);
                }
            }

            let _: () = pipe.query_async(c).await.map_err(GatewayError::from)?;
        }
        Ok(())
    }
}
