use sea_orm::Set;
use sea_orm::entity::prelude::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::utils::error::gateway_error::{GatewayError, Result as GatewayResult};

fn parse_json_field<T: DeserializeOwned>(raw: &str, field: &str) -> GatewayResult<T> {
    serde_json::from_str(raw).map_err(|_| {
        GatewayError::Serialization(format!("Invalid persisted API key JSON field '{field}'"))
    })
}

fn serialize_json_field<T: Serialize>(value: &T, field: &str) -> GatewayResult<String> {
    serde_json::to_string(value).map_err(|_| {
        GatewayError::Serialization(format!("Failed to serialize API key field '{field}'"))
    })
}

/// API key database model
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "api_keys")]
pub struct Model {
    /// API key ID (UUID)
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    /// Display name
    pub name: String,

    /// Hashed secret value
    #[sea_orm(unique)]
    pub key_hash: String,

    /// Prefix shown to users
    pub key_prefix: String,

    /// Optional owner user
    pub user_id: Option<Uuid>,

    /// Optional owner team
    pub team_id: Option<Uuid>,

    /// JSON array serialized as text
    pub permissions: String,

    /// JSON object serialized as text
    pub rate_limits: Option<String>,

    /// Optional expiration timestamp
    pub expires_at: Option<DateTimeWithTimeZone>,

    /// Whether key is active
    pub is_active: bool,

    /// Last usage timestamp
    pub last_used_at: Option<DateTimeWithTimeZone>,

    /// Usage statistics JSON serialized as text
    pub usage_stats: String,

    /// Metadata extra fields JSON serialized as text
    pub extra: Option<String>,

    /// Creation timestamp
    pub created_at: DateTimeWithTimeZone,

    /// Last update timestamp
    pub updated_at: DateTimeWithTimeZone,

    /// Optimistic lock version
    pub version: i32,
}

/// API key entity relations
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    /// API key belongs to a user
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id",
        on_update = "NoAction",
        on_delete = "Restrict"
    )]
    User,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Convert SeaORM model to domain API key model
    pub fn to_domain_api_key(&self) -> GatewayResult<crate::core::models::ApiKey> {
        use crate::core::models::{Metadata, RateLimits, UsageStats};

        let permissions = parse_json_field::<Vec<String>>(&self.permissions, "permissions")?;
        let rate_limits = self
            .rate_limits
            .as_deref()
            .map(|raw| parse_json_field::<RateLimits>(raw, "rate_limits"))
            .transpose()?;
        let usage_stats = parse_json_field::<UsageStats>(&self.usage_stats, "usage_stats")?;
        let extra = match self.extra.as_deref() {
            Some(raw) => parse_json_field::<HashMap<String, serde_json::Value>>(raw, "extra")?,
            None => HashMap::new(),
        };

        let metadata = Metadata {
            id: self.id,
            created_at: self.created_at.naive_utc().and_utc(),
            updated_at: self.updated_at.naive_utc().and_utc(),
            version: self.version as i64,
            extra,
        };

        Ok(crate::core::models::ApiKey {
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
        })
    }

    /// Convert domain API key model to SeaORM active model
    pub fn from_domain_api_key(
        api_key: &crate::core::models::ApiKey,
    ) -> GatewayResult<ActiveModel> {
        let permissions = serialize_json_field(&api_key.permissions, "permissions")?;
        let rate_limits = api_key
            .rate_limits
            .as_ref()
            .map(|limits| serialize_json_field(limits, "rate_limits"))
            .transpose()?;
        let usage_stats = serialize_json_field(&api_key.usage_stats, "usage_stats")?;
        let extra = if api_key.metadata.extra.is_empty() {
            None
        } else {
            Some(serialize_json_field(&api_key.metadata.extra, "extra")?)
        };

        Ok(ActiveModel {
            id: Set(api_key.metadata.id),
            name: Set(api_key.name.clone()),
            key_hash: Set(api_key.key_hash.clone()),
            key_prefix: Set(api_key.key_prefix.clone()),
            user_id: Set(api_key.user_id),
            team_id: Set(api_key.team_id),
            permissions: Set(permissions),
            rate_limits: Set(rate_limits),
            expires_at: Set(api_key.expires_at.map(Into::into)),
            is_active: Set(api_key.is_active),
            last_used_at: Set(api_key.last_used_at.map(Into::into)),
            usage_stats: Set(usage_stats),
            extra: Set(extra),
            created_at: Set(api_key.metadata.created_at.into()),
            updated_at: Set(api_key.metadata.updated_at.into()),
            version: Set(api_key.metadata.version as i32),
        })
    }
}
