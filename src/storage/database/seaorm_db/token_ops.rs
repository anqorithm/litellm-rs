use crate::utils::error::error::{GatewayError, Result};
use sea_orm::*;
use tracing::debug;

use super::super::entities::{self, password_reset_token};
use super::types::SeaOrmDatabase;

impl SeaOrmDatabase {
    /// Store password reset token
    pub async fn store_password_reset_token(
        &self,
        user_id: uuid::Uuid,
        token: &str,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<()> {
        debug!("Storing password reset token for user: {}", user_id);

        // First, delete any existing tokens for this user
        entities::PasswordResetToken::delete_many()
            .filter(password_reset_token::Column::UserId.eq(user_id))
            .exec(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        // Insert new token
        let active_model = password_reset_token::ActiveModel {
            id: NotSet,
            user_id: Set(user_id),
            token: Set(token.to_string()),
            expires_at: Set(expires_at.into()),
            created_at: Set(chrono::Utc::now().into()),
            used_at: Set(None),
        };

        entities::PasswordResetToken::insert(active_model)
            .exec(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        Ok(())
    }

    /// Verify and consume password reset token
    pub async fn verify_password_reset_token(&self, token: &str) -> Result<Option<uuid::Uuid>> {
        debug!("Verifying password reset token");

        let token_model = entities::PasswordResetToken::find()
            .filter(password_reset_token::Column::Token.eq(token))
            .filter(password_reset_token::Column::UsedAt.is_null())
            .filter(password_reset_token::Column::ExpiresAt.gt(chrono::Utc::now()))
            .one(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        if let Some(token_model) = token_model {
            // Mark token as used
            let mut active_model: password_reset_token::ActiveModel = token_model.clone().into();
            active_model.used_at = Set(Some(chrono::Utc::now().into()));

            active_model
                .update(&self.db)
                .await
                .map_err(GatewayError::Database)?;

            Ok(Some(token_model.user_id))
        } else {
            Ok(None)
        }
    }

    /// Invalidate password reset token
    pub async fn invalidate_password_reset_token(&self, token: &str) -> Result<()> {
        debug!("Invalidating password reset token");

        let token_model = entities::PasswordResetToken::find()
            .filter(password_reset_token::Column::Token.eq(token))
            .one(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        if let Some(token_model) = token_model {
            let mut active_model: password_reset_token::ActiveModel = token_model.into();
            active_model.used_at = Set(Some(chrono::Utc::now().into()));

            active_model
                .update(&self.db)
                .await
                .map_err(GatewayError::Database)?;
        }

        Ok(())
    }

    /// Clean up expired password reset tokens
    #[allow(dead_code)] // Reserved for future token cleanup functionality
    pub async fn cleanup_expired_tokens(&self) -> Result<u64> {
        debug!("Cleaning up expired password reset tokens");

        let result = entities::PasswordResetToken::delete_many()
            .filter(password_reset_token::Column::ExpiresAt.lt(chrono::Utc::now()))
            .exec(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        Ok(result.rows_affected)
    }
}
