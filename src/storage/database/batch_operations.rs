//! Batch database operations for improved performance
//!
//! This module provides efficient batch operations for database interactions,
//! reducing the number of round trips and improving overall performance.

use crate::core::models::user::types::User;
use crate::storage::database::Database;
use crate::utils::error::gateway_error::{GatewayError, Result};
use sqlx::{Postgres, Sqlite, Transaction};
use tracing::info;
use uuid::Uuid;

/// Batch size for database operations
const DEFAULT_BATCH_SIZE: usize = 1000;

/// Batch operation configuration
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Maximum batch size
    pub max_batch_size: usize,
    /// Timeout for batch operations
    pub timeout_seconds: u64,
    /// Enable parallel processing
    pub enable_parallel: bool,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: DEFAULT_BATCH_SIZE,
            timeout_seconds: 30,
            enable_parallel: true,
        }
    }
}

/// Batch database operations
pub struct BatchOperations {
    database: Database,
    config: BatchConfig,
}

impl BatchOperations {
    /// Create a new batch operations instance
    pub fn new(database: Database, config: BatchConfig) -> Self {
        Self { database, config }
    }

    /// Batch create users
    pub async fn batch_create_users(&self, users: &[User]) -> Result<Vec<User>> {
        if users.is_empty() {
            return Ok(Vec::new());
        }

        let mut created_users = Vec::with_capacity(users.len());
        
        // Process in batches to avoid overwhelming the database
        for chunk in users.chunks(self.config.max_batch_size) {
            let batch_result = self.create_users_batch(chunk).await?;
            created_users.extend(batch_result);
        }

        info!("Batch created {} users", created_users.len());
        Ok(created_users)
    }

    /// Create a batch of users in a single transaction
    async fn create_users_batch(&self, users: &[User]) -> Result<Vec<User>> {
        match &self.database {
            #[cfg(feature = "postgres")]
            Database::Postgres(pool) => {
                let mut tx = pool.begin().await.map_err(GatewayError::Database)?;
                let mut created_users = Vec::with_capacity(users.len());

                for user in users {
                    let created_user = self.create_user_in_transaction(&mut tx, user).await?;
                    created_users.push(created_user);
                }

                tx.commit().await.map_err(GatewayError::Database)?;
                Ok(created_users)
            }
            #[cfg(feature = "sqlite")]
            Database::Sqlite(pool) => {
                let mut tx = pool.begin().await.map_err(GatewayError::Database)?;
                let mut created_users = Vec::with_capacity(users.len());

                for user in users {
                    let created_user = self.create_user_in_transaction_sqlite(&mut tx, user).await?;
                    created_users.push(created_user);
                }

                tx.commit().await.map_err(GatewayError::Database)?;
                Ok(created_users)
            }
        }
    }

    /// Create a user within a PostgreSQL transaction
    #[cfg(feature = "postgres")]
    async fn create_user_in_transaction(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        user: &User,
    ) -> Result<User> {
        let query = r#"
            INSERT INTO users (id, username, email, password_hash, display_name, role, 
                             email_verified, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
        "#;

        let _row = sqlx::query(query)
            .bind(user.id())
            .bind(&user.username)
            .bind(&user.email)
            .bind(&user.password_hash)
            .bind(&user.display_name)
            .bind(format!("{:?}", user.role))
            .bind(user.email_verified)
            .bind(user.is_active())
            .bind(user.metadata.created_at)
            .bind(user.metadata.updated_at)
            .fetch_one(&mut **tx)
            .await
            .map_err(GatewayError::Database)?;

        // Convert row back to User (simplified for example)
        Ok(user.clone())
    }

    /// Create a user within a SQLite transaction
    #[cfg(feature = "sqlite")]
    async fn create_user_in_transaction_sqlite(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        user: &User,
    ) -> Result<User> {
        let query = r#"
            INSERT INTO users (id, username, email, password_hash, display_name, role, 
                             email_verified, is_active, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        "#;

        sqlx::query(query)
            .bind(user.id().to_string())
            .bind(&user.username)
            .bind(&user.email)
            .bind(&user.password_hash)
            .bind(&user.display_name)
            .bind(format!("{:?}", user.role))
            .bind(user.email_verified)
            .bind(user.is_active())
            .bind(user.metadata.created_at)
            .bind(user.metadata.updated_at)
            .execute(&mut **tx)
            .await
            .map_err(GatewayError::Database)?;

        Ok(user.clone())
    }

    /// Batch update user last login times
    pub async fn batch_update_last_login(&self, user_ids: &[Uuid]) -> Result<u64> {
        if user_ids.is_empty() {
            return Ok(0);
        }

        let now = chrono::Utc::now();
        let mut total_updated = 0;

        // Process in batches
        for chunk in user_ids.chunks(self.config.max_batch_size) {
            let updated = self.update_last_login_batch(chunk, now).await?;
            total_updated += updated;
        }

        info!("Batch updated last login for {} users", total_updated);
        Ok(total_updated)
    }

    /// Update last login for a batch of users
    async fn update_last_login_batch(
        &self,
        user_ids: &[Uuid],
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> Result<u64> {
        match &self.database {
            #[cfg(feature = "postgres")]
            Database::Postgres(pool) => {
                let placeholders: Vec<String> = (1..=user_ids.len())
                    .map(|i| format!("${}", i))
                    .collect();
                
                let query = format!(
                    "UPDATE users SET last_login_at = ${}::timestamptz, updated_at = ${}::timestamptz WHERE id = ANY(ARRAY[{}])",
                    user_ids.len() + 1,
                    user_ids.len() + 2,
                    placeholders.join(", ")
                );

                let mut query_builder = sqlx::query(&query);
                for user_id in user_ids {
                    query_builder = query_builder.bind(user_id);
                }
                query_builder = query_builder.bind(timestamp).bind(timestamp);

                let result = query_builder
                    .execute(pool)
                    .await
                    .map_err(GatewayError::Database)?;

                Ok(result.rows_affected())
            }
            #[cfg(feature = "sqlite")]
            Database::Sqlite(pool) => {
                let placeholders: Vec<String> = user_ids.iter().map(|_| "?".to_string()).collect();
                let query = format!(
                    "UPDATE users SET last_login_at = ?, updated_at = ? WHERE id IN ({})",
                    placeholders.join(", ")
                );

                let mut query_builder = sqlx::query(&query)
                    .bind(timestamp)
                    .bind(timestamp);
                
                for user_id in user_ids {
                    query_builder = query_builder.bind(user_id.to_string());
                }

                let result = query_builder
                    .execute(pool)
                    .await
                    .map_err(GatewayError::Database)?;

                Ok(result.rows_affected())
            }
        }
    }

    /// Batch delete users by IDs
    pub async fn batch_delete_users(&self, user_ids: &[Uuid]) -> Result<u64> {
        if user_ids.is_empty() {
            return Ok(0);
        }

        let mut total_deleted = 0;

        // Process in batches
        for chunk in user_ids.chunks(self.config.max_batch_size) {
            let deleted = self.delete_users_batch(chunk).await?;
            total_deleted += deleted;
        }

        info!("Batch deleted {} users", total_deleted);
        Ok(total_deleted)
    }

    /// Delete a batch of users
    async fn delete_users_batch(&self, user_ids: &[Uuid]) -> Result<u64> {
        match &self.database {
            #[cfg(feature = "postgres")]
            Database::Postgres(pool) => {
                let placeholders: Vec<String> = (1..=user_ids.len())
                    .map(|i| format!("${}", i))
                    .collect();
                
                let query = format!(
                    "DELETE FROM users WHERE id = ANY(ARRAY[{}])",
                    placeholders.join(", ")
                );

                let mut query_builder = sqlx::query(&query);
                for user_id in user_ids {
                    query_builder = query_builder.bind(user_id);
                }

                let result = query_builder
                    .execute(pool)
                    .await
                    .map_err(GatewayError::Database)?;

                Ok(result.rows_affected())
            }
            #[cfg(feature = "sqlite")]
            Database::Sqlite(pool) => {
                let placeholders: Vec<String> = user_ids.iter().map(|_| "?".to_string()).collect();
                let query = format!(
                    "DELETE FROM users WHERE id IN ({})",
                    placeholders.join(", ")
                );

                let mut query_builder = sqlx::query(&query);
                for user_id in user_ids {
                    query_builder = query_builder.bind(user_id.to_string());
                }

                let result = query_builder
                    .execute(pool)
                    .await
                    .map_err(GatewayError::Database)?;

                Ok(result.rows_affected())
            }
        }
    }

    /// Get database statistics
    pub async fn get_stats(&self) -> Result<DatabaseStats> {
        match &self.database {
            #[cfg(feature = "postgres")]
            Database::Postgres(pool) => {
                let user_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
                    .fetch_one(pool)
                    .await
                    .map_err(GatewayError::Database)?;

                Ok(DatabaseStats {
                    total_users: user_count.0 as u64,
                    active_connections: pool.size() as u64,
                    idle_connections: pool.num_idle() as u64,
                })
            }
            #[cfg(feature = "sqlite")]
            Database::Sqlite(pool) => {
                let user_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
                    .fetch_one(pool)
                    .await
                    .map_err(GatewayError::Database)?;

                Ok(DatabaseStats {
                    total_users: user_count.0 as u64,
                    active_connections: pool.size() as u64,
                    idle_connections: pool.num_idle() as u64,
                })
            }
        }
    }
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub total_users: u64,
    pub active_connections: u64,
    pub idle_connections: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_batch_config() {
        let config = BatchConfig::default();
        assert_eq!(config.max_batch_size, DEFAULT_BATCH_SIZE);
        assert_eq!(config.timeout_seconds, 30);
        assert!(config.enable_parallel);
    }

    #[test]
    fn test_database_stats() {
        let stats = DatabaseStats {
            total_users: 1000,
            active_connections: 5,
            idle_connections: 10,
        };

        assert_eq!(stats.total_users, 1000);
        assert_eq!(stats.active_connections, 5);
        assert_eq!(stats.idle_connections, 10);
    }
}
