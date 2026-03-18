use crate::utils::error::gateway_error::{GatewayError, Result};
use sea_orm::prelude::Expr;
use sea_orm::*;
use tracing::debug;

use super::super::entities::{self, api_key};
use super::types::SeaOrmDatabase;

impl SeaOrmDatabase {
    /// Create a new API key
    pub async fn create_api_key(
        &self,
        api_key: &crate::core::models::ApiKey,
    ) -> Result<crate::core::models::ApiKey> {
        debug!("Creating API key: {}", api_key.metadata.id);

        let active_model = api_key::Model::from_domain_api_key(api_key);
        entities::ApiKey::insert(active_model)
            .exec(&self.db)
            .await
            .map_err(GatewayError::from)?;

        Ok(api_key.clone())
    }

    /// Find API key by hash
    pub async fn find_api_key_by_hash(
        &self,
        key_hash: &str,
    ) -> Result<Option<crate::core::models::ApiKey>> {
        debug!("Finding API key by hash");

        let model = entities::ApiKey::find()
            .filter(api_key::Column::KeyHash.eq(key_hash))
            .one(&self.db)
            .await
            .map_err(GatewayError::from)?;

        Ok(model.map(|m| m.to_domain_api_key()))
    }

    /// Find API key by ID
    pub async fn find_api_key_by_id(
        &self,
        key_id: uuid::Uuid,
    ) -> Result<Option<crate::auth::ApiKey>> {
        debug!("Finding API key by ID: {}", key_id);

        let model = entities::ApiKey::find_by_id(key_id)
            .one(&self.db)
            .await
            .map_err(GatewayError::from)?;

        Ok(model.map(|m| m.to_domain_api_key()))
    }

    /// Deactivate API key with transaction wrapping and optimistic locking
    pub async fn deactivate_api_key(&self, key_id: uuid::Uuid) -> Result<()> {
        debug!("Deactivating API key: {}", key_id);

        let txn = self.db.begin().await.map_err(GatewayError::from)?;

        let model = entities::ApiKey::find_by_id(key_id)
            .one(&txn)
            .await
            .map_err(GatewayError::from)?
            .ok_or_else(|| GatewayError::NotFound("API key not found".to_string()))?;

        let current_version = model.version;
        let next_version = current_version + 1;

        let result = entities::ApiKey::update_many()
            .col_expr(api_key::Column::IsActive, Expr::value(false))
            .col_expr(api_key::Column::UpdatedAt, Expr::value(chrono::Utc::now()))
            .col_expr(api_key::Column::Version, Expr::value(next_version))
            .filter(api_key::Column::Id.eq(key_id))
            .filter(api_key::Column::Version.eq(current_version))
            .exec(&txn)
            .await
            .map_err(GatewayError::from)?;

        if result.rows_affected == 0 {
            txn.rollback().await.map_err(GatewayError::from)?;
            return Err(GatewayError::Conflict(
                "API key was modified concurrently".to_string(),
            ));
        }

        txn.commit().await.map_err(GatewayError::from)?;
        Ok(())
    }

    /// List API keys by user
    /// Note: Changed from i64 to Uuid to avoid lossy conversion from Uuid->i64
    pub async fn list_api_keys_by_user(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<Vec<crate::auth::ApiKey>> {
        debug!("Listing API keys for user: {}", user_id);

        let models = entities::ApiKey::find()
            .filter(api_key::Column::UserId.eq(user_id))
            .all(&self.db)
            .await
            .map_err(GatewayError::from)?;

        Ok(models.into_iter().map(|m| m.to_domain_api_key()).collect())
    }

    /// List API keys by team
    pub async fn list_api_keys_by_team(
        &self,
        team_id: uuid::Uuid,
    ) -> Result<Vec<crate::auth::ApiKey>> {
        debug!("Listing API keys for team: {}", team_id);

        let models = entities::ApiKey::find()
            .filter(api_key::Column::TeamId.eq(team_id))
            .all(&self.db)
            .await
            .map_err(GatewayError::from)?;

        Ok(models.into_iter().map(|m| m.to_domain_api_key()).collect())
    }

    /// Update API key permissions with transaction wrapping and optimistic locking
    pub async fn update_api_key_permissions(
        &self,
        key_id: uuid::Uuid,
        permissions: &[String],
    ) -> Result<()> {
        debug!("Updating API key permissions: {}", key_id);

        let txn = self.db.begin().await.map_err(GatewayError::from)?;

        let model = entities::ApiKey::find_by_id(key_id)
            .one(&txn)
            .await
            .map_err(GatewayError::from)?
            .ok_or_else(|| GatewayError::NotFound("API key not found".to_string()))?;

        let serialized = serde_json::to_string(permissions)
            .map_err(|e| GatewayError::Validation(format!("Invalid permissions: {}", e)))?;

        let current_version = model.version;
        let next_version = current_version + 1;

        let result = entities::ApiKey::update_many()
            .col_expr(api_key::Column::Permissions, Expr::value(serialized))
            .col_expr(api_key::Column::UpdatedAt, Expr::value(chrono::Utc::now()))
            .col_expr(api_key::Column::Version, Expr::value(next_version))
            .filter(api_key::Column::Id.eq(key_id))
            .filter(api_key::Column::Version.eq(current_version))
            .exec(&txn)
            .await
            .map_err(GatewayError::from)?;

        if result.rows_affected == 0 {
            txn.rollback().await.map_err(GatewayError::from)?;
            return Err(GatewayError::Conflict(
                "API key was modified concurrently".to_string(),
            ));
        }

        txn.commit().await.map_err(GatewayError::from)?;
        Ok(())
    }

    /// Update API key rate limits with transaction wrapping and optimistic locking
    pub async fn update_api_key_rate_limits(
        &self,
        key_id: uuid::Uuid,
        rate_limits: &crate::core::models::RateLimits,
    ) -> Result<()> {
        debug!("Updating API key rate limits: {}", key_id);

        let txn = self.db.begin().await.map_err(GatewayError::from)?;

        let model = entities::ApiKey::find_by_id(key_id)
            .one(&txn)
            .await
            .map_err(GatewayError::from)?
            .ok_or_else(|| GatewayError::NotFound("API key not found".to_string()))?;

        let serialized = serde_json::to_string(rate_limits)
            .map_err(|e| GatewayError::Validation(format!("Invalid rate limits: {}", e)))?;

        let current_version = model.version;
        let next_version = current_version + 1;

        let result = entities::ApiKey::update_many()
            .col_expr(api_key::Column::RateLimits, Expr::value(Some(serialized)))
            .col_expr(api_key::Column::UpdatedAt, Expr::value(chrono::Utc::now()))
            .col_expr(api_key::Column::Version, Expr::value(next_version))
            .filter(api_key::Column::Id.eq(key_id))
            .filter(api_key::Column::Version.eq(current_version))
            .exec(&txn)
            .await
            .map_err(GatewayError::from)?;

        if result.rows_affected == 0 {
            txn.rollback().await.map_err(GatewayError::from)?;
            return Err(GatewayError::Conflict(
                "API key was modified concurrently".to_string(),
            ));
        }

        txn.commit().await.map_err(GatewayError::from)?;
        Ok(())
    }

    /// Update API key expiration with transaction wrapping and optimistic locking
    pub async fn update_api_key_expiration(
        &self,
        key_id: uuid::Uuid,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<()> {
        debug!("Updating API key expiration: {}", key_id);

        let txn = self.db.begin().await.map_err(GatewayError::from)?;

        let model = entities::ApiKey::find_by_id(key_id)
            .one(&txn)
            .await
            .map_err(GatewayError::from)?
            .ok_or_else(|| GatewayError::NotFound("API key not found".to_string()))?;

        let current_version = model.version;
        let next_version = current_version + 1;

        let result = entities::ApiKey::update_many()
            .col_expr(api_key::Column::ExpiresAt, Expr::value(expires_at))
            .col_expr(api_key::Column::UpdatedAt, Expr::value(chrono::Utc::now()))
            .col_expr(api_key::Column::Version, Expr::value(next_version))
            .filter(api_key::Column::Id.eq(key_id))
            .filter(api_key::Column::Version.eq(current_version))
            .exec(&txn)
            .await
            .map_err(GatewayError::from)?;

        if result.rows_affected == 0 {
            txn.rollback().await.map_err(GatewayError::from)?;
            return Err(GatewayError::Conflict(
                "API key was modified concurrently".to_string(),
            ));
        }

        txn.commit().await.map_err(GatewayError::from)?;
        Ok(())
    }

    /// Update API key usage statistics with transaction wrapping and optimistic locking.
    ///
    /// This is the most critical read-modify-write operation: it reads current usage
    /// stats, computes new totals, and writes them back. Without a transaction and
    /// optimistic lock, concurrent requests can lose usage increments.
    pub async fn update_api_key_usage(
        &self,
        key_id: uuid::Uuid,
        requests: u64,
        tokens: u64,
        cost: f64,
    ) -> Result<()> {
        debug!("Updating API key usage: {}", key_id);

        let txn = self.db.begin().await.map_err(GatewayError::from)?;

        let model = entities::ApiKey::find_by_id(key_id)
            .one(&txn)
            .await
            .map_err(GatewayError::from)?
            .ok_or_else(|| GatewayError::NotFound("API key not found".to_string()))?;

        let mut domain_key = model.to_domain_api_key();
        domain_key.usage_stats.total_requests = domain_key
            .usage_stats
            .total_requests
            .saturating_add(requests);
        domain_key.usage_stats.total_tokens =
            domain_key.usage_stats.total_tokens.saturating_add(tokens);
        domain_key.usage_stats.total_cost += cost;
        domain_key.usage_stats.requests_today = domain_key
            .usage_stats
            .requests_today
            .saturating_add(requests as u32);
        domain_key.usage_stats.tokens_today = domain_key
            .usage_stats
            .tokens_today
            .saturating_add(tokens as u32);
        domain_key.usage_stats.cost_today += cost;

        let usage_stats = serde_json::to_string(&domain_key.usage_stats)
            .map_err(|e| GatewayError::Validation(format!("Invalid usage stats: {}", e)))?;

        let current_version = model.version;
        let next_version = current_version + 1;

        let result = entities::ApiKey::update_many()
            .col_expr(api_key::Column::UsageStats, Expr::value(usage_stats))
            .col_expr(api_key::Column::UpdatedAt, Expr::value(chrono::Utc::now()))
            .col_expr(api_key::Column::Version, Expr::value(next_version))
            .filter(api_key::Column::Id.eq(key_id))
            .filter(api_key::Column::Version.eq(current_version))
            .exec(&txn)
            .await
            .map_err(GatewayError::from)?;

        if result.rows_affected == 0 {
            txn.rollback().await.map_err(GatewayError::from)?;
            return Err(GatewayError::Conflict(
                "API key was modified concurrently".to_string(),
            ));
        }

        txn.commit().await.map_err(GatewayError::from)?;
        Ok(())
    }

    /// Update API key last used timestamp with transaction wrapping and optimistic locking
    pub async fn update_api_key_last_used(&self, key_id: uuid::Uuid) -> Result<()> {
        debug!("Updating API key last_used_at: {}", key_id);

        let txn = self.db.begin().await.map_err(GatewayError::from)?;

        let model = entities::ApiKey::find_by_id(key_id)
            .one(&txn)
            .await
            .map_err(GatewayError::from)?
            .ok_or_else(|| GatewayError::NotFound("API key not found".to_string()))?;

        let current_version = model.version;
        let next_version = current_version + 1;
        let now = chrono::Utc::now();

        let result = entities::ApiKey::update_many()
            .col_expr(api_key::Column::LastUsedAt, Expr::value(Some(now)))
            .col_expr(api_key::Column::UpdatedAt, Expr::value(now))
            .col_expr(api_key::Column::Version, Expr::value(next_version))
            .filter(api_key::Column::Id.eq(key_id))
            .filter(api_key::Column::Version.eq(current_version))
            .exec(&txn)
            .await
            .map_err(GatewayError::from)?;

        if result.rows_affected == 0 {
            txn.rollback().await.map_err(GatewayError::from)?;
            return Err(GatewayError::Conflict(
                "API key was modified concurrently".to_string(),
            ));
        }

        txn.commit().await.map_err(GatewayError::from)?;
        Ok(())
    }

    /// Delete expired API keys
    pub async fn delete_expired_api_keys(&self) -> Result<u64> {
        debug!("Deleting expired API keys");

        let result = entities::ApiKey::delete_many()
            .filter(api_key::Column::ExpiresAt.lt(chrono::Utc::now()))
            .exec(&self.db)
            .await
            .map_err(GatewayError::from)?;

        Ok(result.rows_affected)
    }
}
