//! Password management operations

use super::system::AuthSystem;
use crate::utils::auth::crypto::keys::generate_token;
use crate::utils::auth::crypto::password::{hash_password, verify_password};
use crate::utils::error::gateway_error::{GatewayError, Result};
use tracing::info;
use uuid::Uuid;

impl AuthSystem {
    /// Change user password
    pub async fn change_password(
        &self,
        user_id: Uuid,
        old_password: &str,
        new_password: &str,
    ) -> Result<()> {
        info!("Changing password for user: {}", user_id);

        // Get user
        let user = self
            .storage
            .db()
            .find_user_by_id(user_id)
            .await?
            .ok_or_else(|| GatewayError::not_found("User not found"))?;

        // Verify old password
        if !verify_password(old_password, &user.password_hash)? {
            return Err(GatewayError::auth("Invalid current password"));
        }

        // Hash new password
        let new_password_hash = hash_password(new_password)?;

        // Update password
        self.storage
            .db()
            .update_user_password(user_id, &new_password_hash)
            .await?;

        info!("Password changed successfully for user: {}", user_id);
        Ok(())
    }

    /// Reset password (generate reset token)
    pub async fn request_password_reset(&self, email: &str) -> Result<String> {
        info!("Password reset requested for email: {}", email);

        // Find user by email
        let user = self
            .storage
            .db()
            .find_user_by_email(email)
            .await?
            .ok_or_else(|| GatewayError::not_found("User not found"))?;

        // Generate reset token
        let reset_token = generate_token(32);
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);

        // Store reset token
        self.storage
            .db()
            .store_password_reset_token(user.id(), &reset_token, expires_at)
            .await?;

        info!("Password reset token generated for user: {}", user.id());
        Ok(reset_token)
    }

    /// Reset password using token
    pub async fn reset_password(&self, token: &str, new_password: &str) -> Result<()> {
        info!("Resetting password with token");

        // Verify reset token
        let user_id = self
            .storage
            .db()
            .verify_password_reset_token(token)
            .await?
            .ok_or_else(|| GatewayError::auth("Invalid or expired reset token"))?;

        // Hash new password
        let password_hash = hash_password(new_password)?;

        // Update password
        self.storage
            .db()
            .update_user_password(user_id, &password_hash)
            .await?;

        // Invalidate reset token
        self.storage
            .db()
            .invalidate_password_reset_token(token)
            .await?;

        info!("Password reset successfully for user: {}", user_id);
        Ok(())
    }
}
