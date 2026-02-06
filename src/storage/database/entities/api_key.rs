use sea_orm::Set;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// API key database model
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "api_keys")]
pub struct Model {
    /// API key ID
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    /// Key name/description
    pub name: String,
    /// Hashed key value
    #[sea_orm(unique)]
    pub key_hash: String,
    /// Key prefix
    pub key_prefix: String,
    /// Associated user ID
    pub user_id: Option<Uuid>,
    /// Associated team ID
    pub team_id: Option<Uuid>,
    /// Permissions JSON
    #[sea_orm(column_type = "Json", nullable)]
    pub permissions: Option<serde_json::Value>,
    /// Rate limits JSON
    #[sea_orm(column_type = "Json", nullable)]
    pub rate_limits: Option<serde_json::Value>,
    /// Usage stats JSON
    #[sea_orm(column_type = "Json", nullable)]
    pub usage_stats: Option<serde_json::Value>,
    /// Whether key is active
    pub is_active: bool,
    /// Expiration timestamp
    pub expires_at: Option<DateTimeWithTimeZone>,
    /// Last used timestamp
    pub last_used_at: Option<DateTimeWithTimeZone>,
    /// Created timestamp
    pub created_at: DateTimeWithTimeZone,
    /// Updated timestamp
    pub updated_at: DateTimeWithTimeZone,
    /// Version for optimistic locking
    pub version: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Convert SeaORM model to domain API key model
    pub fn to_domain_api_key(&self) -> crate::core::models::ApiKey {
        use crate::core::models::{Metadata, RateLimits, UsageStats};

        let metadata = Metadata {
            id: self.id,
            created_at: self.created_at.naive_utc().and_utc(),
            updated_at: self.updated_at.naive_utc().and_utc(),
            version: self.version as i64,
            extra: std::collections::HashMap::new(),
        };

        let permissions = self
            .permissions
            .as_ref()
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default();

        let rate_limits = self
            .rate_limits
            .as_ref()
            .and_then(|value| serde_json::from_value::<RateLimits>(value.clone()).ok());

        let usage_stats = self
            .usage_stats
            .as_ref()
            .and_then(|value| serde_json::from_value::<UsageStats>(value.clone()).ok())
            .unwrap_or_default();

        crate::core::models::ApiKey {
            metadata,
            name: self.name.clone(),
            key_hash: self.key_hash.clone(),
            key_prefix: self.key_prefix.clone(),
            user_id: self.user_id,
            team_id: self.team_id,
            permissions,
            rate_limits,
            expires_at: self.expires_at.map(|dt| dt.naive_utc().and_utc()),
            is_active: self.is_active,
            last_used_at: self.last_used_at.map(|dt| dt.naive_utc().and_utc()),
            usage_stats,
        }
    }

    /// Convert domain API key to SeaORM active model
    pub fn from_domain_api_key(api_key: &crate::core::models::ApiKey) -> ActiveModel {
        let permissions = serde_json::to_value(&api_key.permissions)
            .unwrap_or_else(|_| serde_json::json!([]));
        let usage_stats = serde_json::to_value(&api_key.usage_stats)
            .unwrap_or_else(|_| serde_json::json!({}));

        ActiveModel {
            id: Set(api_key.metadata.id),
            name: Set(api_key.name.clone()),
            key_hash: Set(api_key.key_hash.clone()),
            key_prefix: Set(api_key.key_prefix.clone()),
            user_id: Set(api_key.user_id),
            team_id: Set(api_key.team_id),
            permissions: Set(Some(permissions)),
            rate_limits: Set(
                api_key
                    .rate_limits
                    .as_ref()
                    .and_then(|limits| serde_json::to_value(limits).ok()),
            ),
            usage_stats: Set(Some(usage_stats)),
            is_active: Set(api_key.is_active),
            expires_at: Set(api_key.expires_at.map(|dt| dt.into())),
            last_used_at: Set(api_key.last_used_at.map(|dt| dt.into())),
            created_at: Set(api_key.metadata.created_at.into()),
            updated_at: Set(api_key.metadata.updated_at.into()),
            version: Set(api_key.metadata.version as i32),
        }
    }
}
