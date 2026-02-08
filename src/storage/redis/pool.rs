//! Redis connection pool and core connection management
//!
//! This module provides Redis connectivity, connection pooling, and health checks.

use crate::config::RedisConfig;
use crate::utils::error::error::{GatewayError, Result};
use redis::{Client, aio::MultiplexedConnection};
use tracing::{debug, info};

/// Redis connection pool (supports no-op mode when Redis is unavailable)
#[derive(Debug, Clone)]
pub struct RedisPool {
    /// Redis client (None in no-op mode)
    pub(crate) client: Option<Client>,
    /// Connection manager (None in no-op mode)
    pub(crate) connection_manager: Option<MultiplexedConnection>,
    /// Configuration
    pub(crate) config: RedisConfig,
    /// Whether this is a no-op pool (Redis unavailable)
    pub(crate) noop_mode: bool,
}

/// Redis connection wrapper
pub struct RedisConnection {
    pub(crate) conn: Option<MultiplexedConnection>,
}

impl RedisPool {
    /// Create a new Redis pool
    pub async fn new(config: &RedisConfig) -> Result<Self> {
        info!("Creating Redis connection pool");
        debug!("Redis URL: {}", Self::sanitize_url(&config.url));

        let client = Client::open(config.url.as_str()).map_err(GatewayError::Redis)?;

        let connection_manager = client
            .get_multiplexed_async_connection()
            .await
            .map_err(GatewayError::Redis)?;

        info!("Redis connection pool created successfully");
        Ok(Self {
            client: Some(client),
            connection_manager: Some(connection_manager),
            config: config.clone(),
            noop_mode: false,
        })
    }

    /// Create a no-op Redis pool (for when Redis is unavailable)
    pub fn create_noop() -> Self {
        info!("Creating no-op Redis pool (Redis unavailable)");
        Self {
            client: None,
            connection_manager: None,
            config: RedisConfig {
                url: String::new(),
                enabled: false,
                max_connections: 0,
                connection_timeout: 0,
                cluster: false,
            },
            noop_mode: true,
        }
    }

    /// Check if this is a no-op pool
    pub fn is_noop(&self) -> bool {
        self.noop_mode
    }

    /// Get a connection from the pool
    pub async fn get_connection(&self) -> Result<RedisConnection> {
        Ok(RedisConnection {
            conn: self.connection_manager.clone(),
        })
    }

    /// Health check
    pub async fn health_check(&self) -> Result<()> {
        if self.noop_mode {
            debug!("Redis health check skipped (no-op mode)");
            return Ok(());
        }

        debug!("Performing Redis health check");
        let mut conn = self.get_connection().await?;
        if let Some(ref mut c) = conn.conn {
            let _: String = redis::cmd("PING")
                .query_async(c)
                .await
                .map_err(GatewayError::Redis)?;
        }

        debug!("Redis health check passed");
        Ok(())
    }

    /// Close the connection pool
    pub async fn close(&self) -> Result<()> {
        info!("Closing Redis connection pool");
        // Connection manager will be dropped automatically
        info!("Redis connection pool closed");
        Ok(())
    }

    /// Sanitize Redis URL for logging (hide password)
    pub(crate) fn sanitize_url(url: &str) -> String {
        if let Ok(parsed) = url::Url::parse(url) {
            let mut sanitized = parsed.clone();
            if sanitized.password().is_some() {
                let _ = sanitized.set_password(Some("***"));
            }
            sanitized.to_string()
        } else {
            "invalid_url".to_string()
        }
    }
}
