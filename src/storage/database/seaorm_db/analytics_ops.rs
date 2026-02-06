use crate::core::models::metrics::request::{RequestMetrics, RequestStatus};
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

    /// Store request metrics
    #[allow(dead_code)] // Reserved for future metrics storage functionality
    pub async fn store_metrics(
        &self,
        metrics: &RequestMetrics,
    ) -> Result<()> {
        debug!("Storing request metrics: {}", metrics.request_id);

        let status = status_to_string(&metrics.status);
        let error_message = metrics.error.as_ref().map(|err| err.message.clone());

        let active_model = request_log::ActiveModel {
            id: Set(metrics.metadata.id),
            request_id: Set(metrics.request_id.clone()),
            user_id: Set(metrics.user_id),
            team_id: Set(metrics.team_id),
            api_key_id: Set(metrics.api_key_id),
            model: Set(metrics.model.clone()),
            provider: Set(metrics.provider.clone()),
            request_type: Set(metrics.request_type.clone()),
            status: Set(status.to_string()),
            status_code: Set(metrics.status_code as i32),
            input_tokens: Set(metrics.token_usage.input_tokens as i32),
            output_tokens: Set(metrics.token_usage.output_tokens as i32),
            total_tokens: Set(metrics.token_usage.total_tokens as i32),
            input_cost: Set(metrics.cost.input_cost),
            output_cost: Set(metrics.cost.output_cost),
            total_cost: Set(metrics.cost.total_cost),
            response_time_ms: Set(metrics.response_time_ms as i64),
            queue_time_ms: Set(metrics.queue_time_ms as i64),
            provider_time_ms: Set(metrics.provider_time_ms as i64),
            cache_hit: Set(metrics.cache.hit),
            error_message: Set(error_message),
            created_at: Set(metrics.timestamp.into()),
        };

        entities::RequestLog::insert(active_model)
            .exec(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        Ok(())
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

fn status_to_string(status: &RequestStatus) -> &'static str {
    match status {
        RequestStatus::Success => "success",
        RequestStatus::Error => "error",
        RequestStatus::Timeout => "timeout",
        RequestStatus::RateLimit => "rate_limit",
        RequestStatus::QuotaExceeded => "quota_exceeded",
        RequestStatus::Cancelled => "cancelled",
    }
}

#[cfg(test)]
mod tests {
    use super::status_to_string;
    use crate::core::models::metrics::request::RequestStatus;

    #[test]
    fn test_status_to_string() {
        assert_eq!(status_to_string(&RequestStatus::Success), "success");
        assert_eq!(status_to_string(&RequestStatus::Error), "error");
        assert_eq!(status_to_string(&RequestStatus::Timeout), "timeout");
        assert_eq!(status_to_string(&RequestStatus::RateLimit), "rate_limit");
        assert_eq!(status_to_string(&RequestStatus::QuotaExceeded), "quota_exceeded");
        assert_eq!(status_to_string(&RequestStatus::Cancelled), "cancelled");
    }
}
