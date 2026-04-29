//! Virtual key database operations
//!
//! Stores virtual key domain objects as JSON snapshots in the `virtual_keys`
//! table while keeping indexed columns for common lookup and budget queries.

use crate::core::virtual_keys::VirtualKey;
use crate::utils::error::gateway_error::{GatewayError, Result};
use chrono::Utc;
use sea_orm::{ConnectionTrait, DbBackend, Statement, Value};
use tracing::debug;

use super::types::{DatabaseBackendType, SeaOrmDatabase};

impl SeaOrmDatabase {
    fn virtual_key_db_backend(&self) -> DbBackend {
        match self.backend_type {
            DatabaseBackendType::PostgreSQL => DbBackend::Postgres,
            DatabaseBackendType::SQLite => DbBackend::Sqlite,
        }
    }

    fn virtual_key_ph(&self, n: usize) -> String {
        match self.backend_type {
            DatabaseBackendType::PostgreSQL => format!("${}", n),
            DatabaseBackendType::SQLite => "?".to_string(),
        }
    }

    fn serialize_virtual_key(key: &VirtualKey) -> Result<String> {
        serde_json::to_string(key).map_err(|e| GatewayError::Internal(e.to_string()))
    }

    fn deserialize_virtual_key(data: &str) -> Result<VirtualKey> {
        serde_json::from_str(data).map_err(|e| GatewayError::Internal(e.to_string()))
    }

    async fn fetch_virtual_key_data_by_column(
        &self,
        column: &str,
        value: &str,
    ) -> Result<Option<(String, VirtualKey)>> {
        let sql = format!(
            "SELECT data FROM virtual_keys WHERE {} = {}",
            column,
            self.virtual_key_ph(1)
        );
        let stmt = Statement::from_sql_and_values(
            self.virtual_key_db_backend(),
            &sql,
            [Value::String(Some(Box::new(value.to_owned())))],
        );
        match self.db.query_one(stmt).await.map_err(GatewayError::from)? {
            None => Ok(None),
            Some(row) => {
                let data: String = row.try_get("", "data").map_err(GatewayError::from)?;
                let key = Self::deserialize_virtual_key(&data)?;
                Ok(Some((data, key)))
            }
        }
    }

    async fn fetch_virtual_key_by_column(
        &self,
        column: &str,
        value: &str,
    ) -> Result<Option<VirtualKey>> {
        Ok(self
            .fetch_virtual_key_data_by_column(column, value)
            .await?
            .map(|(_, key)| key))
    }

    async fn persist_virtual_key_snapshot(&self, key: &VirtualKey) -> Result<()> {
        let data = Self::serialize_virtual_key(key)?;
        let sql = format!(
            "UPDATE virtual_keys SET user_id = {}, data = {}, spend = {}, budget_reset_at = {}, is_active = {} WHERE key_id = {}",
            self.virtual_key_ph(1),
            self.virtual_key_ph(2),
            self.virtual_key_ph(3),
            self.virtual_key_ph(4),
            self.virtual_key_ph(5),
            self.virtual_key_ph(6),
        );
        let stmt = Statement::from_sql_and_values(
            self.virtual_key_db_backend(),
            &sql,
            [
                Value::String(Some(Box::new(key.user_id.clone()))),
                Value::String(Some(Box::new(data))),
                Value::Double(Some(key.spend)),
                Value::ChronoDateTimeUtc(key.budget_reset_at.map(Box::new)),
                Value::Bool(Some(key.is_active)),
                Value::String(Some(Box::new(key.key_id.clone()))),
            ],
        );
        self.db.execute(stmt).await.map_err(GatewayError::from)?;
        Ok(())
    }

    async fn persist_virtual_key_snapshot_if_data_matches(
        &self,
        key: &VirtualKey,
        expected_data: &str,
    ) -> Result<bool> {
        let data = Self::serialize_virtual_key(key)?;
        let sql = format!(
            "UPDATE virtual_keys SET user_id = {}, data = {}, spend = {}, budget_reset_at = {}, is_active = {} WHERE key_id = {} AND data = {}",
            self.virtual_key_ph(1),
            self.virtual_key_ph(2),
            self.virtual_key_ph(3),
            self.virtual_key_ph(4),
            self.virtual_key_ph(5),
            self.virtual_key_ph(6),
            self.virtual_key_ph(7),
        );
        let stmt = Statement::from_sql_and_values(
            self.virtual_key_db_backend(),
            &sql,
            [
                Value::String(Some(Box::new(key.user_id.clone()))),
                Value::String(Some(Box::new(data))),
                Value::Double(Some(key.spend)),
                Value::ChronoDateTimeUtc(key.budget_reset_at.map(Box::new)),
                Value::Bool(Some(key.is_active)),
                Value::String(Some(Box::new(key.key_id.clone()))),
                Value::String(Some(Box::new(expected_data.to_owned()))),
            ],
        );
        let result = self.db.execute(stmt).await.map_err(GatewayError::from)?;
        Ok(result.rows_affected() == 1)
    }

    /// Store a new virtual key in the database.
    pub async fn store_virtual_key(&self, key: &VirtualKey) -> Result<()> {
        debug!("virtual_keys: store {}", key.key_id);
        let data = Self::serialize_virtual_key(key)?;
        let sql = format!(
            "INSERT INTO virtual_keys (key_id, key_hash, user_id, data, spend, budget_reset_at, is_active) VALUES ({}, {}, {}, {}, {}, {}, {})",
            self.virtual_key_ph(1),
            self.virtual_key_ph(2),
            self.virtual_key_ph(3),
            self.virtual_key_ph(4),
            self.virtual_key_ph(5),
            self.virtual_key_ph(6),
            self.virtual_key_ph(7),
        );
        let stmt = Statement::from_sql_and_values(
            self.virtual_key_db_backend(),
            &sql,
            [
                Value::String(Some(Box::new(key.key_id.clone()))),
                Value::String(Some(Box::new(key.key_hash.clone()))),
                Value::String(Some(Box::new(key.user_id.clone()))),
                Value::String(Some(Box::new(data))),
                Value::Double(Some(key.spend)),
                Value::ChronoDateTimeUtc(key.budget_reset_at.map(Box::new)),
                Value::Bool(Some(key.is_active)),
            ],
        );
        self.db.execute(stmt).await.map_err(GatewayError::from)?;
        Ok(())
    }

    /// Retrieve a virtual key by its hash.
    pub async fn get_virtual_key(&self, key_hash: &str) -> Result<Option<VirtualKey>> {
        debug!("virtual_keys: get by hash");
        self.fetch_virtual_key_by_column("key_hash", key_hash).await
    }

    /// Update the usage statistics (last_used_at, usage_count) for a virtual key.
    pub async fn update_virtual_key_usage(&self, key: &VirtualKey) -> Result<()> {
        debug!("virtual_keys: update usage {}", key.key_id);
        for _ in 0..3 {
            let (old_data, mut updated) = self
                .fetch_virtual_key_data_by_column("key_id", &key.key_id)
                .await?
                .ok_or_else(|| GatewayError::NotFound("Virtual key not found".to_string()))?;

            updated.last_used_at = key.last_used_at.or_else(|| Some(Utc::now()));
            updated.usage_count = updated.usage_count.saturating_add(1);

            if self
                .persist_virtual_key_snapshot_if_data_matches(&updated, &old_data)
                .await?
            {
                return Ok(());
            }
        }

        Err(GatewayError::Conflict(
            "Virtual key usage was modified concurrently".to_string(),
        ))
    }

    /// Add `cost` to the recorded spend for the given key ID.
    pub async fn update_key_spend(&self, key_id: &str, cost: f64) -> Result<()> {
        debug!("virtual_keys: update spend {} += {}", key_id, cost);

        // Optimistic read-modify-write on the JSON snapshot. The data predicate
        // uses the raw stored JSON so concurrent updates cannot be missed due to
        // serialization ordering differences.
        for _ in 0..3 {
            let (old_data, mut updated) = self
                .fetch_virtual_key_data_by_column("key_id", key_id)
                .await?
                .ok_or_else(|| GatewayError::NotFound("Virtual key not found".to_string()))?;
            updated.spend += cost;

            if self
                .persist_virtual_key_snapshot_if_data_matches(&updated, &old_data)
                .await?
            {
                return Ok(());
            }
        }

        Err(GatewayError::Conflict(
            "Virtual key spend was modified concurrently".to_string(),
        ))
    }

    /// List all virtual keys owned by a user.
    pub async fn list_user_keys(&self, user_id: &str) -> Result<Vec<VirtualKey>> {
        debug!("virtual_keys: list user {}", user_id);
        let sql = format!(
            "SELECT data FROM virtual_keys WHERE user_id = {} ORDER BY created_at ASC",
            self.virtual_key_ph(1)
        );
        let stmt = Statement::from_sql_and_values(
            self.virtual_key_db_backend(),
            &sql,
            [Value::String(Some(Box::new(user_id.to_owned())))],
        );
        let rows = self.db.query_all(stmt).await.map_err(GatewayError::from)?;
        rows.into_iter()
            .map(|row| {
                let data: String = row.try_get("", "data").map_err(GatewayError::from)?;
                Self::deserialize_virtual_key(&data)
            })
            .collect()
    }

    /// Retrieve a virtual key by its opaque key ID.
    pub async fn get_virtual_key_by_id(&self, key_id: &str) -> Result<Option<VirtualKey>> {
        debug!("virtual_keys: get by id {}", key_id);
        self.fetch_virtual_key_by_column("key_id", key_id).await
    }

    /// Persist all mutable fields of a virtual key (full update).
    pub async fn update_virtual_key(&self, key: &VirtualKey) -> Result<()> {
        debug!("virtual_keys: update {}", key.key_id);
        self.persist_virtual_key_snapshot(key).await
    }

    /// Remove a virtual key from the database by its key ID.
    pub async fn delete_virtual_key(&self, key_id: &str) -> Result<()> {
        debug!("virtual_keys: delete {}", key_id);
        let sql = format!(
            "DELETE FROM virtual_keys WHERE key_id = {}",
            self.virtual_key_ph(1)
        );
        let stmt = Statement::from_sql_and_values(
            self.virtual_key_db_backend(),
            &sql,
            [Value::String(Some(Box::new(key_id.to_owned())))],
        );
        self.db.execute(stmt).await.map_err(GatewayError::from)?;
        Ok(())
    }

    /// Return all virtual keys whose budget reset timestamp has passed.
    pub async fn get_keys_with_expired_budgets(&self) -> Result<Vec<VirtualKey>> {
        debug!("virtual_keys: get expired budgets");
        let sql = format!(
            "SELECT data FROM virtual_keys WHERE budget_reset_at IS NOT NULL AND budget_reset_at <= {} ORDER BY budget_reset_at ASC",
            self.virtual_key_ph(1)
        );
        let stmt = Statement::from_sql_and_values(
            self.virtual_key_db_backend(),
            &sql,
            [Value::ChronoDateTimeUtc(Some(Box::new(Utc::now())))],
        );
        let rows = self.db.query_all(stmt).await.map_err(GatewayError::from)?;
        rows.into_iter()
            .map(|row| {
                let data: String = row.try_get("", "data").map_err(GatewayError::from)?;
                Self::deserialize_virtual_key(&data)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::models::storage::DatabaseConfig;
    use crate::core::virtual_keys::Permission;
    use std::collections::HashMap;

    fn test_key() -> VirtualKey {
        VirtualKey {
            key_id: "vk-test".to_string(),
            key_hash: "hash-test".to_string(),
            key_alias: Some("test".to_string()),
            user_id: "user-1".to_string(),
            team_id: None,
            organization_id: None,
            models: vec!["gpt-4".to_string()],
            max_budget: Some(100.0),
            spend: 0.0,
            budget_duration: Some("1d".to_string()),
            budget_reset_at: Some(Utc::now() - chrono::Duration::minutes(1)),
            rate_limits: None,
            permissions: vec![Permission::ChatCompletion],
            metadata: HashMap::new(),
            expires_at: None,
            is_active: true,
            created_at: Utc::now(),
            last_used_at: None,
            usage_count: 0,
            tags: vec!["test".to_string()],
        }
    }

    async fn test_db() -> SeaOrmDatabase {
        let db = SeaOrmDatabase::new(&DatabaseConfig::default())
            .await
            .expect("in-memory sqlite should initialize");
        db.migrate().await.expect("migrations should run");
        db
    }

    #[tokio::test]
    async fn virtual_key_crud_round_trip() {
        let db = test_db().await;
        let key = test_key();

        db.store_virtual_key(&key).await.unwrap();
        let by_hash = db.get_virtual_key(&key.key_hash).await.unwrap().unwrap();
        assert_eq!(by_hash.key_id, key.key_id);

        let by_user = db.list_user_keys(&key.user_id).await.unwrap();
        assert_eq!(by_user.len(), 1);

        db.update_key_spend(&key.key_id, 12.5).await.unwrap();

        let mut stale_used_key = by_hash.clone();
        stale_used_key.last_used_at = Some(Utc::now());
        stale_used_key.usage_count = 1;
        db.update_virtual_key_usage(&stale_used_key).await.unwrap();
        let usage_updated = db
            .get_virtual_key_by_id(&key.key_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(usage_updated.usage_count, 1);
        assert_eq!(usage_updated.spend, 12.5);

        let expired = db.get_keys_with_expired_budgets().await.unwrap();
        assert_eq!(expired.len(), 1);

        db.delete_virtual_key(&key.key_id).await.unwrap();
        assert!(
            db.get_virtual_key_by_id(&key.key_id)
                .await
                .unwrap()
                .is_none()
        );
    }
}
