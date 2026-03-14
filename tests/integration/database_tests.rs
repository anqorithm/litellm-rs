#![cfg(feature = "storage")]
//! Database integration tests
//!
//! Tests database operations using real in-memory SQLite database.

#[cfg(test)]
mod tests {
    use litellm_rs::config::models::storage::DatabaseConfig;
    use litellm_rs::core::models::user::types::User;
    use litellm_rs::core::models::{ApiKey, Metadata, RateLimits, UsageStats};
    use litellm_rs::storage::database::{Database, DatabaseBackendType};
    use uuid::Uuid;

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

    #[tokio::test]
    async fn test_database_disabled_uses_in_memory_sqlite() {
        let config = DatabaseConfig {
            url: "postgresql://unreachable-host:5432/unreachable-db".to_string(),
            max_connections: 10,
            connection_timeout: 1,
            ssl: false,
            enabled: false,
        };

        let db = Database::new(&config).await.expect(
            "When database is disabled, runtime should use in-memory SQLite instead of external DB",
        );
        assert_eq!(db.backend_type(), DatabaseBackendType::SQLite);

        db.migrate()
            .await
            .expect("Migration on in-memory DB failed");
        assert!(db.health_check().await.is_ok());
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

    #[tokio::test]
    async fn test_api_key_crud_flow() {
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

        let mut user = User::new(
            "api-key-test-user".to_string(),
            "api-key-test@example.com".to_string(),
            "hashed-password".to_string(),
        );
        user.metadata.id = Uuid::new_v4();
        db.create_user(&user).await.expect("Failed to create user");
        let user_id = user.id();
        let team_id = Uuid::new_v4();
        let key_id = Uuid::new_v4();
        let now = chrono::Utc::now();

        let api_key = ApiKey {
            metadata: Metadata {
                id: key_id,
                created_at: now,
                updated_at: now,
                version: 1,
                extra: std::collections::HashMap::new(),
            },
            name: "integration-key".to_string(),
            key_hash: "hash-integration-key".to_string(),
            key_prefix: "gw-int".to_string(),
            user_id: Some(user_id),
            team_id: Some(team_id),
            permissions: vec!["chat:read".to_string()],
            rate_limits: None,
            expires_at: Some(now + chrono::Duration::days(7)),
            is_active: true,
            last_used_at: None,
            usage_stats: UsageStats {
                last_reset: now,
                ..UsageStats::default()
            },
        };

        let created = db
            .create_api_key(&api_key)
            .await
            .expect("Failed to create api key");
        assert_eq!(created.metadata.id, key_id);

        let by_hash = db
            .find_api_key_by_hash(&api_key.key_hash)
            .await
            .expect("Failed to find api key by hash")
            .expect("API key not found by hash");
        assert_eq!(by_hash.metadata.id, key_id);

        let by_id = db
            .find_api_key_by_id(key_id)
            .await
            .expect("Failed to find api key by id")
            .expect("API key not found by id");
        assert_eq!(by_id.name, "integration-key");

        db.update_api_key_permissions(key_id, &["chat:write".to_string()])
            .await
            .expect("Failed to update permissions");
        let after_permissions = db
            .find_api_key_by_id(key_id)
            .await
            .expect("Failed to refetch api key")
            .expect("API key missing after permissions update");
        assert_eq!(
            after_permissions.permissions,
            vec!["chat:write".to_string()]
        );

        db.update_api_key_rate_limits(
            key_id,
            &RateLimits {
                rpm: Some(60),
                tpm: Some(10000),
                rpd: None,
                tpd: None,
                concurrent: Some(2),
            },
        )
        .await
        .expect("Failed to update rate limits");
        let after_limits = db
            .find_api_key_by_id(key_id)
            .await
            .expect("Failed to refetch api key")
            .expect("API key missing after rate limit update");
        assert_eq!(after_limits.rate_limits.and_then(|r| r.rpm), Some(60));

        db.update_api_key_usage(key_id, 3, 123, 0.42)
            .await
            .expect("Failed to update usage");
        let after_usage = db
            .find_api_key_by_id(key_id)
            .await
            .expect("Failed to refetch api key")
            .expect("API key missing after usage update");
        assert_eq!(after_usage.usage_stats.total_requests, 3);
        assert_eq!(after_usage.usage_stats.total_tokens, 123);

        db.update_api_key_last_used(key_id)
            .await
            .expect("Failed to update last used");
        let after_last_used = db
            .find_api_key_by_id(key_id)
            .await
            .expect("Failed to refetch api key")
            .expect("API key missing after last_used update");
        assert!(after_last_used.last_used_at.is_some());

        db.deactivate_api_key(key_id)
            .await
            .expect("Failed to deactivate key");
        let deactivated = db
            .find_api_key_by_id(key_id)
            .await
            .expect("Failed to refetch api key")
            .expect("API key missing after deactivate");
        assert!(!deactivated.is_active);

        let user_keys = db
            .list_api_keys_by_user(user_id)
            .await
            .expect("Failed to list user api keys");
        assert_eq!(user_keys.len(), 1);

        let team_keys = db
            .list_api_keys_by_team(team_id)
            .await
            .expect("Failed to list team api keys");
        assert_eq!(team_keys.len(), 1);
    }

    #[tokio::test]
    async fn test_api_key_cleanup_expired() {
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

        let now = chrono::Utc::now();

        for (id, expires_at) in [
            (Uuid::new_v4(), Some(now - chrono::Duration::hours(1))),
            (Uuid::new_v4(), Some(now + chrono::Duration::hours(1))),
        ] {
            let api_key = ApiKey {
                metadata: Metadata {
                    id,
                    created_at: now,
                    updated_at: now,
                    version: 1,
                    extra: std::collections::HashMap::new(),
                },
                name: format!("cleanup-{}", id),
                key_hash: format!("hash-{}", id),
                key_prefix: "gw-clean".to_string(),
                user_id: None,
                team_id: None,
                permissions: vec![],
                rate_limits: None,
                expires_at,
                is_active: true,
                last_used_at: None,
                usage_stats: UsageStats {
                    last_reset: now,
                    ..UsageStats::default()
                },
            };
            db.create_api_key(&api_key)
                .await
                .expect("Failed to create api key");
        }

        let deleted = db
            .delete_expired_api_keys()
            .await
            .expect("Failed to clean expired keys");
        assert_eq!(deleted, 1);
    }

    // -------------------------------------------------------------------------
    // Password reset token tests (issue #61 — TOCTOU race condition fix)
    // -------------------------------------------------------------------------

    /// Helper: create an in-memory database with migrations applied.
    async fn make_db() -> Database {
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
        db
    }

    /// Helper: create a user and return its id.
    async fn create_test_user(db: &Database) -> Uuid {
        let mut user = User::new(
            format!("user-{}", Uuid::new_v4()),
            format!("{}@example.com", Uuid::new_v4()),
            "initial-hash".to_string(),
        );
        user.metadata.id = Uuid::new_v4();
        db.create_user(&user).await.expect("Failed to create user");
        user.id()
    }

    /// A valid token allows password reset and is consumed atomically.
    #[tokio::test]
    async fn test_password_reset_token_happy_path() {
        let db = make_db().await;
        let user_id = create_test_user(&db).await;

        let token = "valid-token-abc";
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);
        db.store_password_reset_token(user_id, token, expires_at)
            .await
            .expect("Failed to store token");

        let result = db
            .reset_password_with_token(token, "new-password-hash")
            .await
            .expect("reset_password_with_token failed");
        assert!(result, "expected Ok(true) for a valid token");

        // The token must not be reusable after consumption.
        let second = db
            .reset_password_with_token(token, "another-hash")
            .await
            .expect("second call should not return Err");
        assert!(
            !second,
            "consumed token must not be reusable — got Ok(true) on second call"
        );
    }

    /// An expired token must not allow a password reset.
    #[tokio::test]
    async fn test_password_reset_token_expired() {
        let db = make_db().await;
        let user_id = create_test_user(&db).await;

        let token = "expired-token-xyz";
        // Place the expiry in the past.
        let expires_at = chrono::Utc::now() - chrono::Duration::seconds(1);
        db.store_password_reset_token(user_id, token, expires_at)
            .await
            .expect("Failed to store expired token");

        let result = db
            .reset_password_with_token(token, "some-hash")
            .await
            .expect("reset_password_with_token failed");
        assert!(!result, "expired token must return Ok(false), got Ok(true)");
    }

    /// An unknown / non-existent token must return Ok(false).
    #[tokio::test]
    async fn test_password_reset_token_unknown() {
        let db = make_db().await;

        let result = db
            .reset_password_with_token("non-existent-token", "some-hash")
            .await
            .expect("reset_password_with_token failed");
        assert!(!result, "unknown token must return Ok(false)");
    }

    /// Simulate the TOCTOU scenario: two sequential calls with the same token.
    /// After the first call succeeds the token is consumed; the second call must
    /// fail.  This verifies the core correctness guarantee introduced by the fix
    /// (true concurrent testing would require a multi-connection setup).
    #[tokio::test]
    async fn test_password_reset_token_second_use_rejected() {
        let db = make_db().await;
        let user_id = create_test_user(&db).await;

        let token = "race-token-999";
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);
        db.store_password_reset_token(user_id, token, expires_at)
            .await
            .expect("Failed to store token");

        // First attempt — must succeed.
        let first = db
            .reset_password_with_token(token, "hash-from-first-request")
            .await
            .expect("first call failed");
        assert!(first, "first call with a valid token must succeed");

        // Second attempt with the same token — must be rejected because the
        // token was consumed by the first call inside the transaction.
        let second = db
            .reset_password_with_token(token, "hash-from-second-request")
            .await
            .expect("second call returned Err");
        assert!(
            !second,
            "second call with the same token must return Ok(false) after the token was consumed"
        );

        // Verify the password stored is the one set by the *first* winner.
        let user = db
            .find_user_by_id(user_id)
            .await
            .expect("find_user_by_id failed")
            .expect("user not found");
        assert_eq!(
            user.password_hash, "hash-from-first-request",
            "password must reflect only the first (winning) reset, not the second"
        );
    }
}
