//! SQLite database implementation
//!
//! This module provides SQLite database connectivity and operations.

use crate::config::models::storage::DatabaseConfig;
use crate::utils::error::gateway_error::{GatewayError, Result};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::time::Duration;
use tracing::{debug, info, warn};

/// SQLite database connection pool
#[derive(Debug, Clone)]
pub struct SqliteDatabase {
    pool: SqlitePool,
}

impl SqliteDatabase {
    /// Create a new SQLite database pool
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        info!("Creating SQLite database connection pool");
        debug!("Database URL: {}", Self::sanitize_url(&config.url));

        // Ensure the data directory exists
        if let Some(path) = config.url.strip_prefix("sqlite:") {
            if let Some(parent) = std::path::Path::new(path).parent() {
                tokio::fs::create_dir_all(parent)
                    .await
                    .map_err(|e| GatewayError::Config(format!("Failed to create data directory: {}", e)))?;
            }
        }

        let pool = SqlitePoolOptions::new()
            .max_connections(config.max_connections)
            .acquire_timeout(Duration::from_secs(config.connection_timeout))
            .idle_timeout(Some(Duration::from_secs(600))) // 10 minutes
            .max_lifetime(Some(Duration::from_secs(1800))) // 30 minutes
            .connect(&config.url)
            .await
            .map_err(|e| {
                warn!("Failed to connect to SQLite database: {}", e);
                warn!("Database URL: {}", Self::sanitize_url(&config.url));
                GatewayError::Database(e)
            })?;

        info!("SQLite database connection pool created successfully");
        Ok(Self { pool })
    }

    /// Sanitize URL for logging (remove sensitive information)
    fn sanitize_url(url: &str) -> String {
        if url.starts_with("sqlite:") {
            url.to_string() // SQLite URLs don't contain sensitive info
        } else {
            "***sanitized***".to_string()
        }
    }

    /// Get the underlying pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Execute a query
    pub async fn execute(&self, query: &str) -> Result<u64> {
        let result = sqlx::query(query)
            .execute(&self.pool)
            .await
            .map_err(GatewayError::Database)?;
        Ok(result.rows_affected())
    }

    /// Run database migrations
    pub async fn migrate(&self) -> Result<()> {
        info!("Running SQLite database migrations");
        
        // Create basic tables for SQLite
        self.execute(r#"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                email TEXT UNIQUE NOT NULL,
                display_name TEXT,
                password_hash TEXT NOT NULL,
                role TEXT NOT NULL DEFAULT 'user',
                status TEXT NOT NULL DEFAULT 'pending',
                email_verified BOOLEAN NOT NULL DEFAULT FALSE,
                two_factor_enabled BOOLEAN NOT NULL DEFAULT FALSE,
                last_login_at DATETIME,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
        "#).await?;

        self.execute(r#"
            CREATE TABLE IF NOT EXISTS api_keys (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                name TEXT NOT NULL,
                key_hash TEXT UNIQUE NOT NULL,
                permissions TEXT,
                rate_limit_rpm INTEGER,
                rate_limit_tpm INTEGER,
                expires_at DATETIME,
                last_used_at DATETIME,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (user_id) REFERENCES users(id)
            )
        "#).await?;

        self.execute(r#"
            CREATE TABLE IF NOT EXISTS batches (
                id TEXT PRIMARY KEY,
                object TEXT NOT NULL DEFAULT 'batch',
                endpoint TEXT NOT NULL,
                input_file_id TEXT,
                completion_window TEXT NOT NULL,
                status TEXT NOT NULL,
                output_file_id TEXT,
                error_file_id TEXT,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                in_progress_at DATETIME,
                expires_at DATETIME,
                finalizing_at DATETIME,
                completed_at DATETIME,
                failed_at DATETIME,
                expired_at DATETIME,
                cancelling_at DATETIME,
                cancelled_at DATETIME,
                request_counts_total INTEGER NOT NULL DEFAULT 0,
                request_counts_completed INTEGER NOT NULL DEFAULT 0,
                request_counts_failed INTEGER NOT NULL DEFAULT 0,
                metadata TEXT
            )
        "#).await?;

        info!("SQLite database migrations completed successfully");
        Ok(())
    }

    /// Health check
    pub async fn health_check(&self) -> Result<()> {
        debug!("Performing SQLite database health check");
        
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(GatewayError::Database)?;
        
        debug!("SQLite database health check passed");
        Ok(())
    }

    /// Close the connection pool
    pub async fn close(&self) -> Result<()> {
        info!("Closing SQLite database connection pool");
        self.pool.close().await;
        info!("SQLite database connection pool closed");
        Ok(())
    }
}
