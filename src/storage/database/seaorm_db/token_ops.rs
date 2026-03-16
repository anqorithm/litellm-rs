use crate::utils::error::gateway_error::{GatewayError, Result};
use sea_orm::{sea_query::Expr, *};
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

    /// Check whether a password reset token is valid (unused, not expired)
    /// without consuming it. Used as a cheap pre-validation to avoid
    /// Argon2 CPU cost on invalid tokens.
    pub async fn is_reset_token_valid(&self, token: &str) -> Result<bool> {
        let count = entities::PasswordResetToken::find()
            .filter(password_reset_token::Column::Token.eq(token))
            .filter(password_reset_token::Column::UsedAt.is_null())
            .filter(password_reset_token::Column::ExpiresAt.gt(chrono::Utc::now()))
            .count(&self.db)
            .await
            .map_err(GatewayError::Database)?;
        Ok(count > 0)
    }

    /// Atomically validate, consume a password reset token and update the user's password
    /// in a single database transaction to eliminate the TOCTOU race condition.
    ///
    /// The token is consumed via a single conditional UPDATE statement
    /// (`WHERE used_at IS NULL AND expires_at > now`). Only the request
    /// that gets `rows_affected == 1` proceeds; concurrent requests get
    /// `rows_affected == 0` and return `false` without updating the password.
    ///
    /// Returns `true` if the token was valid and the password was updated,
    /// or `false` if the token was not found, already used, or expired.
    pub async fn reset_password_with_token(
        &self,
        token: &str,
        password_hash: &str,
    ) -> Result<bool> {
        debug!("Atomically consuming password reset token and updating password");

        let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();
        let txn = self.db.begin().await.map_err(GatewayError::Database)?;

        // Single conditional UPDATE: only succeeds if token is unused and not expired.
        // Two concurrent requests cannot both succeed — only one gets rows_affected == 1.
        let update_result = password_reset_token::Entity::update_many()
            .col_expr(password_reset_token::Column::UsedAt, Expr::value(now))
            .filter(password_reset_token::Column::Token.eq(token))
            .filter(password_reset_token::Column::UsedAt.is_null())
            .filter(password_reset_token::Column::ExpiresAt.gt(now))
            .exec(&txn)
            .await
            .map_err(GatewayError::Database)?;

        if update_result.rows_affected == 0 {
            txn.rollback().await.map_err(GatewayError::Database)?;
            return Ok(false);
        }

        // Token is already consumed; fetch user_id — no race possible here.
        let token_model = entities::PasswordResetToken::find()
            .filter(password_reset_token::Column::Token.eq(token))
            .one(&txn)
            .await
            .map_err(GatewayError::Database)?
            .ok_or_else(|| GatewayError::internal("Token disappeared after update"))?;

        let user_id = token_model.user_id;

        // Update the user's password inside the same transaction
        let user_model = entities::User::find_by_id(user_id)
            .one(&txn)
            .await
            .map_err(GatewayError::Database)?
            .ok_or_else(|| GatewayError::NotFound("User not found".to_string()))?;

        let mut user_active: entities::user::ActiveModel = user_model.into();
        user_active.password_hash = Set(password_hash.to_string());
        user_active.updated_at = Set(chrono::Utc::now().into());
        user_active
            .update(&txn)
            .await
            .map_err(GatewayError::Database)?;

        txn.commit().await.map_err(GatewayError::Database)?;

        Ok(true)
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
