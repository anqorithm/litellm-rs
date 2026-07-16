use crate::core::models::user::preferences::UserPreferences;
use crate::core::models::user::types::{User, UserProfile, UserRole as CoreUserRole, UserStatus};
use crate::core::models::{Metadata, UsageStats};
use crate::core::user_management::{
    User as LegacyUser, UserPreferences as LegacyUserPreferences, UserRole as LegacyUserRole,
};
use crate::utils::error::gateway_error::{GatewayError, Result};
use sea_orm::prelude::Expr;
use sea_orm::*;
use tracing::{debug, warn};

use super::super::entities::{self, user};
use super::types::SeaOrmDatabase;

impl SeaOrmDatabase {
    fn core_role_to_legacy(role: &CoreUserRole) -> LegacyUserRole {
        match role {
            CoreUserRole::SuperAdmin => LegacyUserRole::SuperAdmin,
            CoreUserRole::Admin => LegacyUserRole::OrgAdmin,
            CoreUserRole::Manager => LegacyUserRole::TeamAdmin,
            CoreUserRole::User => LegacyUserRole::User,
            CoreUserRole::Viewer => LegacyUserRole::ReadOnly,
            CoreUserRole::ApiUser => LegacyUserRole::ServiceAccount,
        }
    }

    fn legacy_role_to_core(role: &LegacyUserRole) -> CoreUserRole {
        match role {
            LegacyUserRole::SuperAdmin => CoreUserRole::SuperAdmin,
            LegacyUserRole::OrgAdmin => CoreUserRole::Admin,
            LegacyUserRole::TeamAdmin => CoreUserRole::Manager,
            LegacyUserRole::User => CoreUserRole::User,
            LegacyUserRole::ReadOnly => CoreUserRole::Viewer,
            LegacyUserRole::ServiceAccount => CoreUserRole::ApiUser,
        }
    }

    fn core_preferences_to_legacy(preferences: &UserPreferences) -> LegacyUserPreferences {
        LegacyUserPreferences {
            language: preferences.language.clone(),
            timezone: preferences.timezone.clone(),
            email_notifications: preferences.notifications.email_enabled,
            slack_notifications: preferences.notifications.slack_enabled,
            dashboard_config: std::collections::HashMap::new(),
        }
    }

    fn legacy_preferences_to_core(preferences: &LegacyUserPreferences) -> UserPreferences {
        let mut core = UserPreferences {
            language: preferences.language.clone(),
            timezone: preferences.timezone.clone(),
            ..UserPreferences::default()
        };
        core.notifications.email_enabled = preferences.email_notifications;
        core.notifications.slack_enabled = preferences.slack_notifications;
        core
    }

    fn unavailable_legacy_password_hash() -> Result<String> {
        let password = format!("legacy-password-unavailable:{}", uuid::Uuid::new_v4());
        crate::utils::auth::crypto::password::hash_password(&password)
    }

    pub(crate) fn core_user_to_legacy(user: &User) -> LegacyUser {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("canonical_username".to_string(), user.username.clone());

        LegacyUser {
            user_id: user.id().to_string(),
            email: user.email.clone(),
            display_name: user.display_name.clone(),
            first_name: user.profile.first_name.clone(),
            last_name: user.profile.last_name.clone(),
            role: Self::core_role_to_legacy(&user.role),
            teams: user.team_ids.iter().map(ToString::to_string).collect(),
            permissions: Vec::new(),
            metadata,
            max_budget: user
                .rate_limits
                .as_ref()
                .and_then(|limits| limits.monthly_budget),
            spend: user.usage_stats.total_cost,
            budget_duration: None,
            budget_reset_at: None,
            is_active: user.is_active(),
            created_at: user.metadata.created_at,
            last_login_at: user.last_login_at,
            preferences: Self::core_preferences_to_legacy(&user.preferences),
        }
    }

    pub(crate) fn legacy_user_to_core(legacy: &LegacyUser) -> Result<Option<User>> {
        let user_id = match uuid::Uuid::parse_str(&legacy.user_id) {
            Ok(id) => id,
            Err(_) => {
                warn!(
                    legacy_user_id = %legacy.user_id,
                    "Skipping legacy user sync because user_id is not a UUID"
                );
                return Ok(None);
            }
        };

        let mut metadata = Metadata::new();
        metadata.id = user_id;
        metadata.created_at = legacy.created_at;
        metadata.updated_at = legacy.created_at;
        metadata
            .extra
            .insert("legacy_um_user".to_string(), serde_json::Value::Bool(true));

        let team_ids = legacy
            .teams
            .iter()
            .filter_map(|team_id| match uuid::Uuid::parse_str(team_id) {
                Ok(id) => Some(id),
                Err(_) => {
                    warn!(
                        legacy_team_id = %team_id,
                        legacy_user_id = %legacy.user_id,
                        "Skipping legacy user team reference because team_id is not a UUID"
                    );
                    None
                }
            })
            .collect();

        Ok(Some(User {
            metadata,
            username: legacy
                .metadata
                .get("canonical_username")
                .cloned()
                .unwrap_or_else(|| legacy.email.clone()),
            email: legacy.email.clone(),
            display_name: legacy.display_name.clone(),
            password_hash: Self::unavailable_legacy_password_hash()?,
            role: Self::legacy_role_to_core(&legacy.role),
            status: if legacy.is_active {
                UserStatus::Active
            } else {
                UserStatus::Inactive
            },
            team_ids,
            preferences: Self::legacy_preferences_to_core(&legacy.preferences),
            usage_stats: UsageStats {
                total_cost: legacy.spend,
                cost_today: legacy.spend,
                last_reset: legacy.created_at,
                ..UsageStats::default()
            },
            rate_limits: legacy.max_budget.map(|monthly_budget| {
                crate::core::models::user::types::UserRateLimits {
                    rpm: None,
                    tpm: None,
                    rpd: None,
                    tpd: None,
                    concurrent: None,
                    monthly_budget: Some(monthly_budget),
                }
            }),
            last_login_at: legacy.last_login_at,
            email_verified: false,
            two_factor_enabled: false,
            profile: UserProfile {
                first_name: legacy.first_name.clone(),
                last_name: legacy.last_name.clone(),
                ..UserProfile::default()
            },
        }))
    }

    pub(crate) async fn find_canonical_user_by_id(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<Option<User>> {
        let user_model = entities::User::find_by_id(user_id)
            .one(&self.db)
            .await
            .map_err(GatewayError::from)?;

        user_model.map(|model| model.to_domain_user()).transpose()
    }

    pub(crate) async fn find_canonical_user_by_username(
        &self,
        username: &str,
    ) -> Result<Option<User>> {
        let user_model = entities::User::find()
            .filter(user::Column::Username.eq(username))
            .one(&self.db)
            .await
            .map_err(GatewayError::from)?;

        user_model.map(|model| model.to_domain_user()).transpose()
    }

    pub(crate) async fn find_canonical_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let user_model = entities::User::find()
            .filter(user::Column::Email.eq(email))
            .one(&self.db)
            .await
            .map_err(GatewayError::from)?;

        user_model.map(|model| model.to_domain_user()).transpose()
    }

    pub(crate) async fn persist_legacy_user(&self, legacy: &LegacyUser) -> Result<Option<User>> {
        let Some(user) = Self::legacy_user_to_core(legacy)? else {
            return Ok(None);
        };

        if let Some(existing) = self.find_canonical_user_by_id(user.id()).await? {
            if existing.email != user.email {
                warn!(
                    legacy_user_id = %user.id(),
                    legacy_email = %user.email,
                    canonical_email = %existing.email,
                    "Skipping legacy user sync because canonical user ID already exists with a different email"
                );
                return Ok(None);
            }
            return Ok(Some(existing));
        }

        if let Some(existing) = self.find_canonical_user_by_email(&user.email).await?
            && existing.id() != user.id()
        {
            warn!(
                legacy_user_id = %user.id(),
                existing_user_id = %existing.id(),
                email = %user.email,
                "Skipping legacy user sync because canonical user email already exists"
            );
            return Ok(None);
        }

        if let Some(existing) = self.find_canonical_user_by_username(&user.username).await?
            && existing.id() != user.id()
        {
            warn!(
                legacy_user_id = %user.id(),
                existing_user_id = %existing.id(),
                username = %user.username,
                "Skipping legacy user sync because canonical username already exists"
            );
            return Ok(None);
        }

        let active_model = user::Model::from_domain_user(&user);
        entities::User::insert(active_model)
            .exec(&self.db)
            .await
            .map_err(GatewayError::from)?;
        Ok(Some(user))
    }

    pub(crate) async fn sync_legacy_user_from_canonical(&self, user_id: uuid::Uuid) -> Result<()> {
        let Some(user) = self.find_canonical_user_by_id(user_id).await? else {
            return Ok(());
        };
        self.upsert_legacy_user_from_core(&user).await
    }

    async fn upsert_legacy_user_from_core(&self, user: &User) -> Result<()> {
        let legacy = Self::core_user_to_legacy(user);
        if self.get_legacy_user_by_id(&legacy.user_id).await?.is_some() {
            self.update_user(&legacy).await
        } else if let Some(existing) = self.get_legacy_user_by_email(&legacy.email).await? {
            warn!(
                canonical_user_id = %legacy.user_id,
                existing_legacy_user_id = %existing.user_id,
                email = %legacy.email,
                "Skipping canonical user sync because legacy user email already exists"
            );
            Ok(())
        } else {
            self.um_create_user(&legacy).await
        }
    }

    /// Find user by ID
    pub async fn find_user_by_id(&self, user_id: uuid::Uuid) -> Result<Option<User>> {
        debug!("Finding user by ID: {}", user_id);

        if let Some(user) = self.find_canonical_user_by_id(user_id).await? {
            return Ok(Some(user));
        }

        match self.get_user(&user_id.to_string()).await? {
            Some(legacy) => self.persist_legacy_user(&legacy).await,
            None => Ok(None),
        }
    }

    /// Find user by username
    pub async fn find_user_by_username(&self, username: &str) -> Result<Option<User>> {
        debug!("Finding user by username: {}", username);

        if let Some(user) = self.find_canonical_user_by_username(username).await? {
            return Ok(Some(user));
        }

        if let Some(legacy) = self.get_legacy_user_by_canonical_username(username).await? {
            return self.persist_legacy_user(&legacy).await;
        }

        match self.get_user_by_email(username).await? {
            Some(legacy) => self.persist_legacy_user(&legacy).await,
            None => Ok(None),
        }
    }

    /// Find user by email
    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<User>> {
        debug!("Finding user by email: {}", email);

        if let Some(user) = self.find_canonical_user_by_email(email).await? {
            return Ok(Some(user));
        }

        match self.get_user_by_email(email).await? {
            Some(legacy) => self.persist_legacy_user(&legacy).await,
            None => Ok(None),
        }
    }

    /// Create a new user
    pub async fn create_user(&self, user: &User) -> Result<User> {
        debug!("Creating user: {}", user.username);

        let active_model = user::Model::from_domain_user(user);

        let _result = entities::User::insert(active_model)
            .exec(&self.db)
            .await
            .map_err(GatewayError::from)?;

        self.upsert_legacy_user_from_core(user).await?;

        // Return the created user
        Ok(user.clone())
    }

    /// Update user password with transaction wrapping and optimistic locking
    pub async fn update_user_password(
        &self,
        user_id: uuid::Uuid,
        password_hash: &str,
    ) -> Result<()> {
        debug!("Updating password for user: {}", user_id);

        let txn = self.db.begin().await.map_err(GatewayError::from)?;

        let user_model = entities::User::find_by_id(user_id)
            .one(&txn)
            .await
            .map_err(GatewayError::from)?
            .ok_or_else(|| GatewayError::NotFound("User not found".to_string()))?;

        let current_version = user_model.version;
        let next_version = current_version + 1;

        let result = entities::User::update_many()
            .col_expr(
                user::Column::PasswordHash,
                Expr::value(password_hash.to_string()),
            )
            .col_expr(user::Column::UpdatedAt, Expr::value(chrono::Utc::now()))
            .col_expr(user::Column::Version, Expr::value(next_version))
            .filter(user::Column::Id.eq(user_id))
            .filter(user::Column::Version.eq(current_version))
            .exec(&txn)
            .await
            .map_err(GatewayError::from)?;

        if result.rows_affected == 0 {
            txn.rollback().await.map_err(GatewayError::from)?;
            return Err(GatewayError::Conflict(
                "User was modified concurrently".to_string(),
            ));
        }

        txn.commit().await.map_err(GatewayError::from)?;
        Ok(())
    }

    /// Update user last login with transaction wrapping and optimistic locking
    pub async fn update_user_last_login(&self, user_id: uuid::Uuid) -> Result<()> {
        debug!("Updating last login for user: {}", user_id);

        let txn = self.db.begin().await.map_err(GatewayError::from)?;

        let user_model = entities::User::find_by_id(user_id)
            .one(&txn)
            .await
            .map_err(GatewayError::from)?
            .ok_or_else(|| GatewayError::NotFound("User not found".to_string()))?;

        let current_version = user_model.version;
        let next_version = current_version + 1;
        let now = chrono::Utc::now();

        let result = entities::User::update_many()
            .col_expr(user::Column::LastLoginAt, Expr::value(Some(now)))
            .col_expr(user::Column::UpdatedAt, Expr::value(now))
            .col_expr(user::Column::Version, Expr::value(next_version))
            .filter(user::Column::Id.eq(user_id))
            .filter(user::Column::Version.eq(current_version))
            .exec(&txn)
            .await
            .map_err(GatewayError::from)?;

        if result.rows_affected == 0 {
            txn.rollback().await.map_err(GatewayError::from)?;
            return Err(GatewayError::Conflict(
                "User was modified concurrently".to_string(),
            ));
        }

        txn.commit().await.map_err(GatewayError::from)?;
        Ok(())
    }

    /// Verify user email with transaction wrapping and optimistic locking
    pub async fn verify_user_email(&self, user_id: uuid::Uuid) -> Result<()> {
        debug!("Verifying email for user: {}", user_id);

        let txn = self.db.begin().await.map_err(GatewayError::from)?;

        let user_model = entities::User::find_by_id(user_id)
            .one(&txn)
            .await
            .map_err(GatewayError::from)?
            .ok_or_else(|| GatewayError::NotFound("User not found".to_string()))?;

        let current_version = user_model.version;
        let next_version = current_version + 1;

        let result = entities::User::update_many()
            .col_expr(user::Column::EmailVerified, Expr::value(true))
            .col_expr(user::Column::UpdatedAt, Expr::value(chrono::Utc::now()))
            .col_expr(user::Column::Version, Expr::value(next_version))
            .filter(user::Column::Id.eq(user_id))
            .filter(user::Column::Version.eq(current_version))
            .exec(&txn)
            .await
            .map_err(GatewayError::from)?;

        if result.rows_affected == 0 {
            txn.rollback().await.map_err(GatewayError::from)?;
            return Err(GatewayError::Conflict(
                "User was modified concurrently".to_string(),
            ));
        }

        txn.commit().await.map_err(GatewayError::from)?;
        Ok(())
    }
}
