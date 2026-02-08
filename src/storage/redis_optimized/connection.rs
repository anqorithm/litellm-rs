//! Connection management for optimized Redis pool
//!
//! Handles connection creation, pooling, and lifecycle management.

use super::types::{PoolConfig, PooledConnection};
use crate::utils::error::error::{GatewayError, Result};
use redis::{Client, aio::MultiplexedConnection};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, Semaphore};
use tracing::{debug, info, warn};

/// Connection pool manager
pub(super) struct ConnectionPool {
    /// Redis client for creating new connections
    pub(super) client: Client,
    /// Pool of available connections
    pub(super) connections: Arc<RwLock<Vec<PooledConnection>>>,
    /// Semaphore for limiting concurrent connections
    pub(super) semaphore: Arc<Semaphore>,
    /// Pool configuration
    pub(super) pool_config: PoolConfig,
}

impl ConnectionPool {
    /// Create a new connection pool
    pub(super) fn new(client: Client, pool_config: PoolConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(pool_config.max_connections));
        let connections = Arc::new(RwLock::new(Vec::new()));

        Self {
            client,
            connections,
            semaphore,
            pool_config,
        }
    }

    /// Initialize minimum connections in the pool
    pub(super) async fn initialize_connections(&self) -> Result<()> {
        let mut connections = self.connections.write().await;

        for _ in 0..self.pool_config.min_connections {
            match self.create_connection().await {
                Ok(conn) => connections.push(PooledConnection::new(conn)),
                Err(e) => {
                    warn!("Failed to create initial connection: {}", e);
                    break;
                }
            }
        }

        info!("Initialized {} connections", connections.len());
        Ok(())
    }

    /// Create a new Redis connection
    pub(super) async fn create_connection(&self) -> Result<MultiplexedConnection> {
        let connection = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(GatewayError::Redis)?;

        Ok(connection)
    }

    /// Get a connection from the pool
    pub(super) async fn get_connection(&self) -> Result<MultiplexedConnection> {
        // Acquire semaphore permit - handle closed semaphore gracefully
        let _permit = self.semaphore.acquire().await.map_err(|e| {
            GatewayError::Internal(format!("Connection pool semaphore closed: {}", e))
        })?;

        // Try to get an existing connection
        {
            let mut connections = self.connections.write().await;
            if let Some(mut pooled_conn) = connections.pop() {
                if pooled_conn.is_healthy && !pooled_conn.is_idle(self.pool_config.max_idle_time) {
                    pooled_conn.mark_used();
                    return Ok(pooled_conn.connection);
                }
            }
        }

        // Create a new connection if none available
        self.create_connection().await
    }

    /// Return a connection to the pool for reuse
    pub(super) async fn return_connection(&self, connection: MultiplexedConnection) {
        let mut connections = self.connections.write().await;

        if connections.len() < self.pool_config.max_connections {
            connections.push(PooledConnection::new(connection));
        }
        // If pool is full, connection will be dropped
    }

    /// Start health checker background task
    ///
    /// Periodically removes unhealthy or idle connections from the pool.
    pub(super) fn start_health_checker(&self) {
        let connections = self.connections.clone();
        let interval = self.pool_config.health_check_interval;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;

                let mut conns = connections.write().await;
                conns.retain(|conn| {
                    // Remove unhealthy or idle connections
                    conn.is_healthy && !conn.is_idle(Duration::from_secs(600)) // 10 minutes max idle
                });
            }
        });
    }

    /// Start connection manager background task
    ///
    /// Ensures minimum connections are maintained in the pool.
    pub(super) fn start_connection_manager(&self) {
        let connections = self.connections.clone();
        let client = self.client.clone();
        let min_connections = self.pool_config.min_connections;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(Duration::from_secs(60));

            loop {
                interval_timer.tick().await;

                let mut conns = connections.write().await;

                // Ensure minimum connections
                while conns.len() < min_connections {
                    match client.get_multiplexed_async_connection().await {
                        Ok(conn) => {
                            conns.push(PooledConnection::new(conn));
                            debug!("Added connection to maintain minimum pool size");
                        }
                        Err(e) => {
                            warn!("Failed to create connection for pool maintenance: {}", e);
                            break;
                        }
                    }
                }
            }
        });
    }

    /// Get available permits in the semaphore
    pub(super) fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }
}
