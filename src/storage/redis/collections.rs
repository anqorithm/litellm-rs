//! Redis collection operations (Lists and Sets)
//!
//! This module provides operations for Redis Lists and Sets data structures.

use super::pool::RedisPool;
use crate::utils::error::gateway_error::{GatewayError, Result};
use redis::{AsyncCommands, RedisResult};

impl RedisPool {
    // ===== List operations =====

    /// Push value to list (left push)
    pub async fn list_push(&self, key: &str, value: &str) -> Result<()> {
        if self.noop_mode {
            return Ok(());
        }

        let mut conn = self.get_connection().await?;
        if let Some(ref mut c) = conn.conn {
            let _: () = c.lpush(key, value).await.map_err(GatewayError::from)?;
        }
        Ok(())
    }

    /// Pop value from list (right pop)
    pub async fn list_pop(&self, key: &str) -> Result<Option<String>> {
        if self.noop_mode {
            return Ok(None);
        }

        let mut conn = self.get_connection().await?;
        if let Some(ref mut c) = conn.conn {
            let result: RedisResult<String> = c.rpop(key, None).await;
            match result {
                Ok(value) => Ok(Some(value)),
                Err(e) if e.kind() == redis::ErrorKind::UnexpectedReturnType => Ok(None),
                Err(e) => Err(GatewayError::from(e)),
            }
        } else {
            Ok(None)
        }
    }

    /// Get list length
    pub async fn list_length(&self, key: &str) -> Result<usize> {
        if self.noop_mode {
            return Ok(0);
        }

        let mut conn = self.get_connection().await?;
        if let Some(ref mut c) = conn.conn {
            let len: usize = c.llen(key).await.map_err(GatewayError::from)?;
            Ok(len)
        } else {
            Ok(0)
        }
    }

    /// Get list range
    pub async fn list_range(&self, key: &str, start: isize, stop: isize) -> Result<Vec<String>> {
        if self.noop_mode {
            return Ok(vec![]);
        }

        let mut conn = self.get_connection().await?;
        if let Some(ref mut c) = conn.conn {
            let values: Vec<String> = c
                .lrange(key, start, stop)
                .await
                .map_err(GatewayError::from)?;
            Ok(values)
        } else {
            Ok(vec![])
        }
    }

    // ===== Set operations =====

    /// Add member to set
    pub async fn set_add(&self, key: &str, member: &str) -> Result<()> {
        if self.noop_mode {
            return Ok(());
        }

        let mut conn = self.get_connection().await?;
        if let Some(ref mut c) = conn.conn {
            let _: () = c.sadd(key, member).await.map_err(GatewayError::from)?;
        }
        Ok(())
    }

    /// Remove member from set
    pub async fn set_remove(&self, key: &str, member: &str) -> Result<()> {
        if self.noop_mode {
            return Ok(());
        }

        let mut conn = self.get_connection().await?;
        if let Some(ref mut c) = conn.conn {
            let _: () = c.srem(key, member).await.map_err(GatewayError::from)?;
        }
        Ok(())
    }

    /// Get all set members
    pub async fn set_members(&self, key: &str) -> Result<Vec<String>> {
        if self.noop_mode {
            return Ok(vec![]);
        }

        let mut conn = self.get_connection().await?;
        if let Some(ref mut c) = conn.conn {
            let members: Vec<String> = c.smembers(key).await.map_err(GatewayError::from)?;
            Ok(members)
        } else {
            Ok(vec![])
        }
    }

    /// Check if member is in set
    pub async fn set_is_member(&self, key: &str, member: &str) -> Result<bool> {
        if self.noop_mode {
            return Ok(false);
        }

        let mut conn = self.get_connection().await?;
        if let Some(ref mut c) = conn.conn {
            let is_member: bool = c.sismember(key, member).await.map_err(GatewayError::from)?;
            Ok(is_member)
        } else {
            Ok(false)
        }
    }
}
