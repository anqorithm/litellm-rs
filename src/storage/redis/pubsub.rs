//! Redis Pub/Sub operations
//!
//! This module provides publish/subscribe messaging functionality.
//! Note: Subscription functionality is temporarily disabled due to Redis API changes.

use super::pool::RedisPool;
use crate::utils::error::gateway_error::{GatewayError, Result};
use redis::AsyncCommands;

/// Redis subscription wrapper
/// Note: Subscription functionality temporarily disabled due to Redis API changes
/// This should be fixed when updating to a compatible Redis version
#[allow(dead_code)]
pub struct Subscription {
    _placeholder: (),
}

impl RedisPool {
    /// Publish message to channel
    pub async fn publish(&self, channel: &str, message: &str) -> Result<()> {
        if self.noop_mode {
            return Ok(());
        }

        let mut conn = self.get_connection().await?;
        if let Some(ref mut c) = conn.conn {
            let _: () = c
                .publish(channel, message)
                .await
                .map_err(GatewayError::from)?;
        }
        Ok(())
    }

    /// Subscribe to Redis channels for pub/sub messaging
    /// Note: Temporarily disabled due to Redis API compatibility issues
    pub async fn subscribe(&self, _channels: &[String]) -> Result<Subscription> {
        // TODO: Fix when Redis API is updated to compatible version
        Err(GatewayError::Storage(
            "PubSub temporarily disabled due to API compatibility".to_string(),
        ))
    }
}

impl Subscription {
    /// Get the next message
    /// Note: Temporarily disabled due to Redis API compatibility issues
    pub async fn next_message(&mut self) -> Result<redis::Msg> {
        // TODO: Fix when Redis API is updated to compatible version
        Err(GatewayError::Storage(
            "PubSub temporarily disabled due to API compatibility".to_string(),
        ))
    }

    /// Unsubscribe from all channels
    pub async fn unsubscribe_all(&mut self) -> Result<()> {
        // Note: Redis 0.24 doesn't have unsubscribe_all, we'll need to track channels manually
        // For now, just return Ok
        // self.pubsub.unsubscribe_all().await.map_err(GatewayError::from)?;
        Ok(())
    }
}
