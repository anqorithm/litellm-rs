#![cfg(feature = "storage")]
//! Database integration tests
//!
//! Tests database operations using real in-memory SQLite database.

#[cfg(test)]
mod tests {
    use litellm_rs::config::DatabaseConfig;
    use litellm_rs::storage::database::Database;

    /// Test basic database connection and health check
    #[tokio::test]
    async fn test_database_health_check() {
        let config = DatabaseConfig {
            url: "sqlite::memory:".to_string(),
            max_connections: 1,
            connection_timeout: 5,
            ssl: false,
            enabled: true,
        };

        let db = Database::new(&config).await;
        assert!(db.is_ok(), "Failed to create database: {:?}", db.err());

        let db = db.unwrap();

        // Run migrations first to create required tables
        let migrate_result = db.migrate().await;
        assert!(
            migrate_result.is_ok(),
            "Migration failed: {:?}",
            migrate_result.err()
        );

        let health = db.health_check().await;
        assert!(health.is_ok(), "Health check failed: {:?}", health.err());
    }

    /// Test database migration
    #[tokio::test]
    async fn test_database_migration() {
        let config = DatabaseConfig {
            url: "sqlite::memory:".to_string(),
            max_connections: 1,
            connection_timeout: 5,
            ssl: false,
            enabled: true,
        };

        let db = Database::new(&config)
            .await
            .expect("Failed to create database");
        let result = db.migrate().await;
        assert!(result.is_ok(), "Migration failed: {:?}", result.err());
    }

    /// Test database user operations (find_user_by_email, etc.)
    #[tokio::test]
    async fn test_user_operations() {
        let config = DatabaseConfig {
            url: "sqlite::memory:".to_string(),
            max_connections: 1,
            connection_timeout: 5,
            ssl: false,
            enabled: true,
        };

        let db = Database::new(&config)
            .await
            .expect("Failed to create database");
        db.migrate().await.expect("Migration failed");

        // Try to find a user that doesn't exist
        let user = db.find_user_by_email("nonexistent@example.com").await;
        assert!(user.is_ok());
        assert!(user.unwrap().is_none());
    }

    /// Test database batch operations
    #[tokio::test]
    async fn test_batch_list() {
        let config = DatabaseConfig {
            url: "sqlite::memory:".to_string(),
            max_connections: 1,
            connection_timeout: 5,
            ssl: false,
            enabled: true,
        };

        let db = Database::new(&config)
            .await
            .expect("Failed to create database");
        db.migrate().await.expect("Migration failed");

        // List batches (should be empty)
        let batches = db.list_batches(Some(10), None).await;
        assert!(batches.is_ok());
        assert!(batches.unwrap().is_empty());
    }

    /// Test database statistics
    #[tokio::test]
    async fn test_database_stats() {
        let config = DatabaseConfig {
            url: "sqlite::memory:".to_string(),
            max_connections: 1,
            connection_timeout: 5,
            ssl: false,
            enabled: true,
        };

        let db = Database::new(&config)
            .await
            .expect("Failed to create database");
        let stats = db.stats();

        // Just verify we can get stats (size is always >= 0 as usize)
        let _ = stats.size;
    }
}
