use sea_orm::Set;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// User database model
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    /// User ID (UUID)
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    /// Username (unique)
    #[sea_orm(unique)]
    pub username: String,

    /// Email address (unique)
    #[sea_orm(unique)]
    pub email: String,

    /// Password hash
    pub password_hash: String,

    /// Display name (optional)
    pub display_name: Option<String>,

    /// User role
    pub role: String,

    /// User status
    pub status: String,

    /// Email verification status
    pub email_verified: bool,

    /// Two-factor authentication enabled
    pub two_factor_enabled: bool,

    /// Last login timestamp
    pub last_login_at: Option<DateTimeWithTimeZone>,

    /// Creation timestamp
    pub created_at: DateTimeWithTimeZone,

    /// Last update timestamp
    pub updated_at: DateTimeWithTimeZone,

    /// Version for optimistic locking
    pub version: i32,
}

/// User entity relations
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    /// Password reset tokens relation
    #[sea_orm(has_many = "super::password_reset_token::Entity")]
    PasswordResetTokens,

    /// User sessions relation
    #[sea_orm(has_many = "super::user_session::Entity")]
    UserSessions,
}

impl Related<super::password_reset_token::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PasswordResetTokens.def()
    }
}

impl Related<super::user_session::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserSessions.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// Conversion methods between SeaORM model and our domain model
impl Model {
    /// Convert SeaORM model to domain user model
    pub fn to_domain_user(
        &self,
    ) -> crate::utils::error::gateway_error::Result<crate::core::models::user::types::User> {
        use crate::core::models::user::preferences::UserPreferences;
        use crate::core::models::user::types::{UserProfile, UserRole, UserStatus};
        use crate::core::models::{Metadata, UsageStats};
        use crate::utils::error::gateway_error::GatewayError;
        use std::str::FromStr;

        let metadata = Metadata {
            id: self.id,
            created_at: self.created_at.naive_utc().and_utc(),
            updated_at: self.updated_at.naive_utc().and_utc(),
            version: self.version as i64,
            extra: std::collections::HashMap::new(),
        };

        let role = UserRole::from_str(&self.role)
            .map_err(|_| GatewayError::Storage("Invalid persisted users.role enum".to_string()))?;
        let status = match self.status.as_str() {
            "active" => UserStatus::Active,
            "inactive" => UserStatus::Inactive,
            "pending" => UserStatus::Pending,
            "suspended" => UserStatus::Suspended,
            "deleted" => UserStatus::Deleted,
            _ => {
                return Err(GatewayError::Storage(
                    "Invalid persisted users.status enum".to_string(),
                ));
            }
        };

        Ok(crate::core::models::user::types::User {
            metadata,
            username: self.username.clone(),
            email: self.email.clone(),
            display_name: self.display_name.clone(),
            password_hash: self.password_hash.clone(),
            role,
            status,
            team_ids: vec![],
            preferences: UserPreferences::default(),
            usage_stats: UsageStats::default(),
            rate_limits: None,
            last_login_at: self.last_login_at.map(|dt| dt.naive_utc().and_utc()),
            email_verified: self.email_verified,
            two_factor_enabled: self.two_factor_enabled,
            profile: UserProfile::default(),
        })
    }

    /// Convert domain user model to SeaORM active model
    pub fn from_domain_user(user: &crate::core::models::user::types::User) -> ActiveModel {
        ActiveModel {
            id: Set(user.metadata.id),
            username: Set(user.username.clone()),
            email: Set(user.email.clone()),
            password_hash: Set(user.password_hash.clone()),
            display_name: Set(user.display_name.clone()),
            role: Set(user.role.to_string()),
            status: Set(match user.status {
                crate::core::models::user::types::UserStatus::Active => "active".to_string(),
                crate::core::models::user::types::UserStatus::Inactive => "inactive".to_string(),
                crate::core::models::user::types::UserStatus::Pending => "pending".to_string(),
                crate::core::models::user::types::UserStatus::Suspended => "suspended".to_string(),
                crate::core::models::user::types::UserStatus::Deleted => "deleted".to_string(),
            }),
            email_verified: Set(user.email_verified),
            two_factor_enabled: Set(user.two_factor_enabled),
            last_login_at: Set(user.last_login_at.map(|dt| dt.into())),
            created_at: Set(user.metadata.created_at.into()),
            updated_at: Set(user.metadata.updated_at.into()),
            version: Set(user.metadata.version as i32),
        }
    }
}
