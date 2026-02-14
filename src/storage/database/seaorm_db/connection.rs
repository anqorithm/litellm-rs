use crate::config::models::storage::DatabaseConfig;
use crate::utils::error::gateway_error::{GatewayError, Result};
use sea_orm::*;
use sea_orm_migration::MigratorTrait;
use std::time::Duration;
use tracing::{debug, info, warn};

use super::super::entities;
use super::super::migration::Migrator;
use super::types::{DatabaseBackendType, SeaOrmDatabase};

impl SeaOrmDatabase {
    /// Create a new database connection with automatic SQLite fallback
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        // Try primary database connection first
        match Self::try_connect(&config.url, config).await {
            Ok(db) => {
                let backend_type = if config.url.starts_with("sqlite") {
                    DatabaseBackendType::SQLite
                } else {
                    DatabaseBackendType::PostgreSQL
                };
                info!("Database connection established ({:?})", backend_type);
                Ok(Self { db, backend_type })
            }
            Err(e) => {
                // If PostgreSQL connection fails, try SQLite fallback
                if config.url.starts_with("postgresql://") || config.url.starts_with("postgres://")
                {
                    warn!(
                        "PostgreSQL connection failed: {}. Attempting SQLite fallback...",
                        e
                    );
                    Self::fallback_to_sqlite().await
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Try to connect to a database
    async fn try_connect(url: &str, config: &DatabaseConfig) -> Result<DatabaseConnection> {
        let mut opt = ConnectOptions::new(url.to_string());
        opt.max_connections(config.max_connections)
            .min_connections(1)
            .connect_timeout(Duration::from_secs(config.connection_timeout))
            .acquire_timeout(Duration::from_secs(30))
            .idle_timeout(Duration::from_secs(600))
            .max_lifetime(Duration::from_secs(3600))
            .sqlx_logging(false);

        Database::connect(opt).await.map_err(GatewayError::Database)
    }

    /// Fallback to SQLite database
    async fn fallback_to_sqlite() -> Result<Self> {
        // Create data directory if it doesn't exist
        let data_dir = std::path::Path::new("data");
        if !data_dir.exists() {
            std::fs::create_dir_all(data_dir).map_err(|e| {
                GatewayError::Internal(format!("Failed to create data directory: {}", e))
            })?;
        }

        let sqlite_path = "sqlite://data/gateway.db?mode=rwc";
        info!("Falling back to SQLite database: {}", sqlite_path);

        let mut opt = ConnectOptions::new(sqlite_path.to_string());
        opt.max_connections(5)
            .min_connections(1)
            .connect_timeout(Duration::from_secs(5))
            .acquire_timeout(Duration::from_secs(30))
            .idle_timeout(Duration::from_secs(600))
            .max_lifetime(Duration::from_secs(3600))
            .sqlx_logging(false);

        let db = Database::connect(opt)
            .await
            .map_err(GatewayError::Database)?;

        info!("SQLite fallback connection established successfully");
        Ok(Self {
            db,
            backend_type: DatabaseBackendType::SQLite,
        })
    }

    /// Get the current backend type
    pub fn backend_type(&self) -> DatabaseBackendType {
        self.backend_type
    }

    /// Check if using SQLite fallback
    pub fn is_sqlite_fallback(&self) -> bool {
        self.backend_type == DatabaseBackendType::SQLite
    }

    /// Run database migrations
    pub async fn migrate(&self) -> Result<()> {
        info!("Running database migrations...");
        Migrator::up(&self.db, None).await.map_err(|e| {
            warn!("Migration failed: {}", e);
            GatewayError::Database(e)
        })?;
        info!("Database migrations completed successfully");
        Ok(())
    }

    /// Get the underlying database connection
    #[allow(dead_code)] // Reserved for future direct database access
    pub fn connection(&self) -> &DatabaseConnection {
        &self.db
    }

    /// Close the database connection
    #[allow(dead_code)] // Reserved for future connection cleanup
    pub async fn close(self) -> Result<()> {
        self.db.close().await.map_err(GatewayError::Database)?;
        Ok(())
    }

    /// Health check
    pub async fn health_check(&self) -> Result<()> {
        debug!("Performing database health check");

        // Simple query to check database connectivity
        let _result = entities::User::find()
            .limit(1)
            .all(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        debug!("Database health check passed");
        Ok(())
    }
}
