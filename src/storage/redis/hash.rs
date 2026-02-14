//! Redis Hash and Sorted Set operations
//!
//! This module provides operations for Redis Hash and Sorted Set data structures.

use super::pool::RedisPool;
use crate::utils::error::gateway_error::{GatewayError, Result};
use redis::{AsyncCommands, RedisResult};
use std::collections::HashMap;

impl RedisPool {
    // ===== Hash operations =====

    /// Set hash field value
    pub async fn hash_set(&self, key: &str, field: &str, value: &str) -> Result<()> {
        if self.noop_mode {
            return Ok(());
        }

        let mut conn = self.get_connection().await?;
        if let Some(ref mut c) = conn.conn {
            let _: () = c
                .hset(key, field, value)
                .await
                .map_err(GatewayError::Redis)?;
        }
        Ok(())
    }

    /// Get hash field value
    pub async fn hash_get(&self, key: &str, field: &str) -> Result<Option<String>> {
        if self.noop_mode {
            return Ok(None);
        }

        let mut conn = self.get_connection().await?;
        if let Some(ref mut c) = conn.conn {
            let result: RedisResult<String> = c.hget(key, field).await;
            match result {
                Ok(value) => Ok(Some(value)),
                Err(e) if e.kind() == redis::ErrorKind::TypeError => Ok(None),
                Err(e) => Err(GatewayError::Redis(e)),
            }
        } else {
            Ok(None)
        }
    }

    /// Delete hash field
    pub async fn hash_delete(&self, key: &str, field: &str) -> Result<()> {
        if self.noop_mode {
            return Ok(());
        }

        let mut conn = self.get_connection().await?;
        if let Some(ref mut c) = conn.conn {
            let _: () = c.hdel(key, field).await.map_err(GatewayError::Redis)?;
        }
        Ok(())
    }

    /// Get all hash fields and values
    pub async fn hash_get_all(&self, key: &str) -> Result<HashMap<String, String>> {
        if self.noop_mode {
            return Ok(HashMap::new());
        }

        let mut conn = self.get_connection().await?;
        if let Some(ref mut c) = conn.conn {
            let hash: HashMap<String, String> =
                c.hgetall(key).await.map_err(GatewayError::Redis)?;
            Ok(hash)
        } else {
            Ok(HashMap::new())
        }
    }

    /// Check if a hash field exists
    pub async fn hash_exists(&self, key: &str, field: &str) -> Result<bool> {
        if self.noop_mode {
            return Ok(false);
        }

        let mut conn = self.get_connection().await?;
        if let Some(ref mut c) = conn.conn {
            let exists: bool = c.hexists(key, field).await.map_err(GatewayError::Redis)?;
            Ok(exists)
        } else {
            Ok(false)
        }
    }

    // ===== Sorted Set operations =====

    /// Add member to sorted set with score
    pub async fn sorted_set_add(&self, key: &str, score: f64, member: &str) -> Result<()> {
        if self.noop_mode {
            return Ok(());
        }

        let mut conn = self.get_connection().await?;
        if let Some(ref mut c) = conn.conn {
            let _: () = c
                .zadd(key, score, member)
                .await
                .map_err(GatewayError::Redis)?;
        }
        Ok(())
    }

    /// Get a range of elements from a sorted set
    pub async fn sorted_set_range(
        &self,
        key: &str,
        start: isize,
        stop: isize,
    ) -> Result<Vec<String>> {
        if self.noop_mode {
            return Ok(vec![]);
        }

        let mut conn = self.get_connection().await?;
        if let Some(ref mut c) = conn.conn {
            let members: Vec<String> = c
                .zrange(key, start, stop)
                .await
                .map_err(GatewayError::Redis)?;
            Ok(members)
        } else {
            Ok(vec![])
        }
    }

    /// Remove a member from a sorted set
    pub async fn sorted_set_remove(&self, key: &str, member: &str) -> Result<()> {
        if self.noop_mode {
            return Ok(());
        }

        let mut conn = self.get_connection().await?;
        if let Some(ref mut c) = conn.conn {
            let _: () = c.zrem(key, member).await.map_err(GatewayError::Redis)?;
        }
        Ok(())
    }
}
