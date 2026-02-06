use crate::core::models::{ApiKey, RateLimits, UsageStats};
use crate::utils::error::error::{GatewayError, Result};
use chrono::Utc;
use sea_orm::*;
use tracing::debug;

use super::super::entities::{self, api_key};
use super::types::SeaOrmDatabase;

impl SeaOrmDatabase {
    /// Create a new API key
    pub async fn create_api_key(&self, api_key: &ApiKey) -> Result<ApiKey> {
        debug!("Creating API key: {}", api_key.metadata.id);

        let active_model = api_key::Model::from_domain_api_key(api_key);
        entities::ApiKey::insert(active_model)
            .exec(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        Ok(api_key.clone())
    }

    /// Find API key by hash
    pub async fn find_api_key_by_hash(&self, key_hash: &str) -> Result<Option<ApiKey>> {
        debug!("Finding API key by hash");

        let key_model = entities::ApiKey::find()
            .filter(api_key::Column::KeyHash.eq(key_hash))
            .one(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        Ok(key_model.map(|model| model.to_domain_api_key()))
    }

    /// Find API key by ID
    pub async fn find_api_key_by_id(&self, key_id: uuid::Uuid) -> Result<Option<crate::auth::ApiKey>> {
        debug!("Finding API key by ID: {}", key_id);

        let key_model = entities::ApiKey::find_by_id(key_id)
            .one(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        Ok(key_model.map(|model| model.to_domain_api_key()))
    }

    /// Deactivate API key
    pub async fn deactivate_api_key(&self, key_id: uuid::Uuid) -> Result<()> {
        debug!("Deactivating API key: {}", key_id);

        let key_model = entities::ApiKey::find_by_id(key_id)
            .one(&self.db)
            .await
            .map_err(GatewayError::Database)?
            .ok_or_else(|| GatewayError::NotFound("API key not found".to_string()))?;

        let next_version = key_model.version + 1;
        let mut active_model: api_key::ActiveModel = key_model.into();
        active_model.is_active = Set(false);
        active_model.updated_at = Set(Utc::now().into());
        active_model.version = Set(next_version);

        active_model
            .update(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        Ok(())
    }

    /// List API keys by user
    /// Note: Changed from i64 to Uuid to avoid lossy conversion from Uuid->i64
    pub async fn list_api_keys_by_user(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<Vec<crate::auth::ApiKey>> {
        debug!("Listing API keys for user: {}", user_id);

        let key_models = entities::ApiKey::find()
            .filter(api_key::Column::UserId.eq(user_id))
            .all(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        Ok(key_models
            .into_iter()
            .map(|model| model.to_domain_api_key())
            .collect())
    }

    /// List API keys by team
    pub async fn list_api_keys_by_team(
        &self,
        team_id: uuid::Uuid,
    ) -> Result<Vec<crate::auth::ApiKey>> {
        debug!("Listing API keys for team: {}", team_id);

        let key_models = entities::ApiKey::find()
            .filter(api_key::Column::TeamId.eq(team_id))
            .all(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        Ok(key_models
            .into_iter()
            .map(|model| model.to_domain_api_key())
            .collect())
    }

    /// Update API key permissions
    pub async fn update_api_key_permissions(
        &self,
        key_id: uuid::Uuid,
        permissions: &[String],
    ) -> Result<()> {
        debug!("Updating API key permissions: {}", key_id);

        let key_model = entities::ApiKey::find_by_id(key_id)
            .one(&self.db)
            .await
            .map_err(GatewayError::Database)?
            .ok_or_else(|| GatewayError::NotFound("API key not found".to_string()))?;

        let next_version = key_model.version + 1;
        let mut active_model: api_key::ActiveModel = key_model.into();
        active_model.permissions = Set(Some(
            serde_json::to_value(permissions).unwrap_or_else(|_| serde_json::json!([])),
        ));
        active_model.updated_at = Set(Utc::now().into());
        active_model.version = Set(next_version);

        active_model
            .update(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        Ok(())
    }

    /// Update API key rate limits
    pub async fn update_api_key_rate_limits(
        &self,
        key_id: uuid::Uuid,
        rate_limits: &RateLimits,
    ) -> Result<()> {
        debug!("Updating API key rate limits: {}", key_id);

        let key_model = entities::ApiKey::find_by_id(key_id)
            .one(&self.db)
            .await
            .map_err(GatewayError::Database)?
            .ok_or_else(|| GatewayError::NotFound("API key not found".to_string()))?;

        let next_version = key_model.version + 1;
        let mut active_model: api_key::ActiveModel = key_model.into();
        active_model.rate_limits = Set(serde_json::to_value(rate_limits).ok());
        active_model.updated_at = Set(Utc::now().into());
        active_model.version = Set(next_version);

        active_model
            .update(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        Ok(())
    }

    /// Update API key expiration
    pub async fn update_api_key_expiration(
        &self,
        key_id: uuid::Uuid,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<()> {
        debug!("Updating API key expiration: {}", key_id);

        let key_model = entities::ApiKey::find_by_id(key_id)
            .one(&self.db)
            .await
            .map_err(GatewayError::Database)?
            .ok_or_else(|| GatewayError::NotFound("API key not found".to_string()))?;

        let next_version = key_model.version + 1;
        let mut active_model: api_key::ActiveModel = key_model.into();
        active_model.expires_at = Set(expires_at.map(|dt| dt.into()));
        active_model.updated_at = Set(Utc::now().into());
        active_model.version = Set(next_version);

        active_model
            .update(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        Ok(())
    }

    /// Update API key usage statistics
    pub async fn update_api_key_usage(
        &self,
        key_id: uuid::Uuid,
        requests: u64,
        tokens: u64,
        cost: f64,
    ) -> Result<()> {
        debug!("Updating API key usage: {}", key_id);

        let key_model = entities::ApiKey::find_by_id(key_id)
            .one(&self.db)
            .await
            .map_err(GatewayError::Database)?
            .ok_or_else(|| GatewayError::NotFound("API key not found".to_string()))?;

        let current_stats = key_model
            .usage_stats
            .as_ref()
            .and_then(|value| serde_json::from_value::<UsageStats>(value.clone()).ok())
            .unwrap_or_default();

        let updated_stats = update_usage_stats(current_stats, requests, tokens, cost, Utc::now());

        let next_version = key_model.version + 1;
        let mut active_model: api_key::ActiveModel = key_model.into();
        active_model.usage_stats = Set(Some(
            serde_json::to_value(updated_stats).unwrap_or_else(|_| serde_json::json!({})),
        ));
        active_model.updated_at = Set(Utc::now().into());
        active_model.version = Set(next_version);

        active_model
            .update(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        Ok(())
    }

    /// Update API key last used timestamp
    pub async fn update_api_key_last_used(&self, key_id: uuid::Uuid) -> Result<()> {
        debug!("Updating API key last used: {}", key_id);

        let key_model = entities::ApiKey::find_by_id(key_id)
            .one(&self.db)
            .await
            .map_err(GatewayError::Database)?
            .ok_or_else(|| GatewayError::NotFound("API key not found".to_string()))?;

        let next_version = key_model.version + 1;
        let mut active_model: api_key::ActiveModel = key_model.into();
        active_model.last_used_at = Set(Some(Utc::now().into()));
        active_model.updated_at = Set(Utc::now().into());
        active_model.version = Set(next_version);

        active_model
            .update(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        Ok(())
    }

    /// Delete expired API keys
    pub async fn delete_expired_api_keys(&self) -> Result<u64> {
        debug!("Deleting expired API keys");

        let now = Utc::now();
        let result = entities::ApiKey::delete_many()
            .filter(api_key::Column::ExpiresAt.is_not_null())
            .filter(api_key::Column::ExpiresAt.lt(now))
            .exec(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        Ok(result.rows_affected)
    }
}

fn update_usage_stats(
    mut stats: UsageStats,
    requests: u64,
    tokens: u64,
    cost: f64,
    now: chrono::DateTime<chrono::Utc>,
) -> UsageStats {
    stats.total_requests += requests;
    stats.total_tokens += tokens;
    stats.total_cost += cost;

    let last_reset = stats.last_reset.date_naive();
    let today = now.date_naive();
    if last_reset != today {
        stats.requests_today = 0;
        stats.tokens_today = 0;
        stats.cost_today = 0.0;
        stats.last_reset = now;
    }

    stats.requests_today = stats.requests_today.saturating_add(requests as u32);
    stats.tokens_today = stats.tokens_today.saturating_add(tokens as u32);
    stats.cost_today += cost;

    stats
}

#[cfg(test)]
mod tests {
    use super::update_usage_stats;
    use crate::core::models::UsageStats;
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_update_usage_stats_resets_daily() {
        let mut stats = UsageStats::default();
        stats.last_reset = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();

        let now = Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap();
        let updated = update_usage_stats(stats, 5, 100, 0.25, now);

        assert_eq!(updated.requests_today, 5);
        assert_eq!(updated.tokens_today, 100);
        assert!((updated.cost_today - 0.25).abs() < f64::EPSILON);
    }

    #[test]
    fn test_update_usage_stats_accumulates_same_day() {
        let mut stats = UsageStats::default();
        stats.last_reset = Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap();

        let now = Utc.with_ymd_and_hms(2024, 1, 2, 12, 0, 0).unwrap();
        let updated = update_usage_stats(stats, 2, 50, 0.10, now);

        assert_eq!(updated.requests_today, 2);
        assert_eq!(updated.tokens_today, 50);
        assert!((updated.cost_today - 0.10).abs() < f64::EPSILON);
        assert_eq!(updated.total_requests, 2);
        assert_eq!(updated.total_tokens, 50);
    }
}
