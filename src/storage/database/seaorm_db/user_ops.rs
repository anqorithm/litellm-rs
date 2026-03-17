use crate::core::models::user::types::User;
use crate::utils::error::gateway_error::{GatewayError, Result};
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

    /// Update user password
    pub async fn update_user_password(
        &self,
        user_id: uuid::Uuid,
        password_hash: &str,
    ) -> Result<()> {
        debug!("Updating password for user: {}", user_id);

        let mut user: user::ActiveModel = entities::User::find_by_id(user_id)
            .one(&self.db)
            .await
            .map_err(GatewayError::from)?
            .ok_or_else(|| GatewayError::NotFound("User not found".to_string()))?
            .into();

        user.password_hash = Set(password_hash.to_string());
        user.updated_at = Set(chrono::Utc::now().into());

        user.update(&self.db).await.map_err(GatewayError::from)?;

        Ok(())
    }

    /// Update user last login
    pub async fn update_user_last_login(&self, user_id: uuid::Uuid) -> Result<()> {
        debug!("Updating last login for user: {}", user_id);

        let user_model = entities::User::find_by_id(user_id)
            .one(&self.db)
            .await
            .map_err(GatewayError::from)?
            .ok_or_else(|| GatewayError::NotFound("User not found".to_string()))?;

        let mut active_model: user::ActiveModel = user_model.into();
        active_model.last_login_at = Set(Some(chrono::Utc::now().into()));
        active_model.updated_at = Set(chrono::Utc::now().into());

        active_model
            .update(&self.db)
            .await
            .map_err(GatewayError::from)?;

        Ok(())
    }

    /// Verify user email
    pub async fn verify_user_email(&self, user_id: uuid::Uuid) -> Result<()> {
        debug!("Verifying email for user: {}", user_id);

        let user_model = entities::User::find_by_id(user_id)
            .one(&self.db)
            .await
            .map_err(GatewayError::from)?
            .ok_or_else(|| GatewayError::NotFound("User not found".to_string()))?;

        let mut active_model: user::ActiveModel = user_model.into();
        active_model.email_verified = Set(true);
        active_model.updated_at = Set(chrono::Utc::now().into());

        active_model
            .update(&self.db)
            .await
            .map_err(GatewayError::from)?;

        Ok(())
    }
}
