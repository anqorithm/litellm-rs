//! API key management operations

use super::system::AuthSystem;
use crate::core::models::ApiKey;
use crate::utils::error::gateway_error::Result;
use tracing::info;
use uuid::Uuid;

impl AuthSystem {
    /// Create API key for user
    pub async fn create_api_key(
        &self,
        user_id: Uuid,
        name: String,
        permissions: Vec<String>,
    ) -> Result<(ApiKey, String)> {
        info!("Creating API key for user: {}", user_id);
        self.api_key
            .create_key(Some(user_id), None, name, permissions)
            .await
    }

    /// Revoke API key
    pub async fn revoke_api_key(&self, key_id: Uuid) -> Result<()> {
        info!("Revoking API key: {}", key_id);
        self.api_key.revoke_key(key_id).await
    }
}
