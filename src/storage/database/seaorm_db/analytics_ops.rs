use crate::utils::error::error::{GatewayError, Result};
use sea_orm::*;
use tracing::debug;

use super::super::entities::{self, request_log};
use super::types::{DatabaseStats, SeaOrmDatabase};

impl SeaOrmDatabase {
    /// Get user usage statistics
    pub async fn get_user_usage(
        &self,
        user_id: &str,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<serde_json::Value>> {
        debug!("Fetching usage data for user: {}", user_id);

        let user_id = uuid::Uuid::parse_str(user_id).map_err(|e| {
            GatewayError::Validation(format!("Invalid user_id format: {}", e))
        })?;

        let logs = entities::RequestLog::find()
            .filter(request_log::Column::UserId.eq(user_id))
            .filter(request_log::Column::CreatedAt.gte(start))
            .filter(request_log::Column::CreatedAt.lte(end))
            .all(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        let usage = logs
            .into_iter()
            .map(|log| {
                serde_json::json!({
                    "total_tokens": log.total_tokens.max(0) as u64,
                    "cost": log.total_cost,
                    "created_at": log.created_at,
                })
            })
            .collect();

        Ok(usage)
    }
    /// Get database statistics
    pub async fn stats(&self) -> DatabaseStats {
        let total_users = entities::User::find()
            .count(&self.db)
            .await
            .unwrap_or(0);

        // Note: SeaORM doesn't expose pool stats; return 0 for size/idle for now.
        DatabaseStats {
            total_users,
            size: 0,
            idle: 0,
        }
    }
}
