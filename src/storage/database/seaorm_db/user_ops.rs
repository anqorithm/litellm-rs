use crate::core::models::user::types::User;
use crate::utils::error::gateway_error::{GatewayError, Result};
use sea_orm::prelude::Expr;
use sea_orm::*;
use tracing::debug;

use super::super::entities::{self, user};
use super::types::SeaOrmDatabase;

impl SeaOrmDatabase {
    /// Find user by ID
    pub async fn find_user_by_id(&self, user_id: uuid::Uuid) -> Result<Option<User>> {
        debug!("Finding user by ID: {}", user_id);

        let user_model = entities::User::find_by_id(user_id)
            .one(&self.db)
            .await
            .map_err(GatewayError::from)?;

        Ok(user_model.map(|model| model.to_domain_user()))
    }

    /// Find user by username
    pub async fn find_user_by_username(&self, username: &str) -> Result<Option<User>> {
        debug!("Finding user by username: {}", username);

        let user_model = entities::User::find()
            .filter(user::Column::Username.eq(username))
            .one(&self.db)
            .await
            .map_err(GatewayError::from)?;

        Ok(user_model.map(|model| model.to_domain_user()))
    }

    /// Find user by email
    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<User>> {
        debug!("Finding user by email: {}", email);

        let user_model = entities::User::find()
            .filter(user::Column::Email.eq(email))
            .one(&self.db)
            .await
            .map_err(GatewayError::from)?;

        Ok(user_model.map(|model| model.to_domain_user()))
    }

    /// Create a new user
    pub async fn create_user(&self, user: &User) -> Result<User> {
        debug!("Creating user: {}", user.username);

        let active_model = user::Model::from_domain_user(user);

        let _result = entities::User::insert(active_model)
            .exec(&self.db)
            .await
            .map_err(GatewayError::from)?;

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
