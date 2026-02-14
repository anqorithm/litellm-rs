use crate::utils::error::gateway_error::{GatewayError, Result};
use sea_orm::*;
use tracing::{debug, warn};

use super::super::entities;
use super::types::SeaOrmDatabase;

impl SeaOrmDatabase {
    /// Create a new batch
    pub async fn create_batch(&self, batch: &crate::core::batch::BatchRequest) -> Result<String> {
        debug!("Creating batch: {}", batch.batch_id);

        let active_model = entities::batch::ActiveModel {
            id: Set(batch.batch_id.clone()),
            object: Set("batch".to_string()),
            endpoint: Set(match batch.batch_type {
                crate::core::batch::BatchType::ChatCompletion => "/v1/chat/completions".to_string(),
                crate::core::batch::BatchType::Embedding => "/v1/embeddings".to_string(),
                crate::core::batch::BatchType::ImageGeneration => {
                    "/v1/images/generations".to_string()
                }
                crate::core::batch::BatchType::Custom(ref endpoint) => endpoint.clone(),
            }),
            input_file_id: Set(None),
            completion_window: Set(format!("{}h", batch.completion_window.unwrap_or(24))),
            status: Set("validating".to_string()),
            output_file_id: Set(None),
            error_file_id: Set(None),
            created_at: Set(chrono::Utc::now().into()),
            in_progress_at: Set(None),
            finalizing_at: Set(None),
            completed_at: Set(None),
            failed_at: Set(None),
            expired_at: Set(None),
            cancelling_at: Set(None),
            cancelled_at: Set(None),
            request_counts_total: Set(Some(batch.requests.len() as i32)),
            request_counts_completed: Set(Some(0)),
            request_counts_failed: Set(Some(0)),
            metadata: Set(Some(
                serde_json::to_string(&batch.metadata).unwrap_or_default(),
            )),
        };

        entities::Batch::insert(active_model)
            .exec(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        Ok(batch.batch_id.clone())
    }

    /// Update batch status
    pub async fn update_batch_status(&self, batch_id: &str, status: &str) -> Result<()> {
        debug!("Updating batch status: {} -> {}", batch_id, status);

        let batch_model = entities::Batch::find_by_id(batch_id)
            .one(&self.db)
            .await
            .map_err(GatewayError::Database)?
            .ok_or_else(|| GatewayError::NotFound("Batch not found".to_string()))?;

        let mut active_model: entities::batch::ActiveModel = batch_model.into();
        active_model.status = Set(status.to_string());

        // Update timestamp based on status
        let now = chrono::Utc::now().into();
        match status {
            "in_progress" => active_model.in_progress_at = Set(Some(now)),
            "finalizing" => active_model.finalizing_at = Set(Some(now)),
            "completed" => active_model.completed_at = Set(Some(now)),
            "failed" => active_model.failed_at = Set(Some(now)),
            "expired" => active_model.expired_at = Set(Some(now)),
            "cancelling" => active_model.cancelling_at = Set(Some(now)),
            "cancelled" => active_model.cancelled_at = Set(Some(now)),
            _ => {}
        }

        active_model
            .update(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        Ok(())
    }

    /// List batches with pagination
    pub async fn list_batches(
        &self,
        limit: Option<i32>,
        after: Option<&str>,
    ) -> Result<Vec<crate::core::batch::BatchRecord>> {
        debug!(
            "Listing batches with limit: {:?}, after: {:?}",
            limit, after
        );

        let mut query = entities::Batch::find();

        if let Some(after_id) = after {
            query = query.filter(entities::batch::Column::Id.gt(after_id));
        }

        if let Some(limit) = limit {
            query = query.limit(limit as u64);
        }

        let batch_models = query
            .order_by_desc(entities::batch::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        let batch_records = batch_models
            .into_iter()
            .map(|model| {
                // Parse status string to BatchStatus enum
                let status = match model.status.as_str() {
                    "validating" => crate::core::batch::BatchStatus::Validating,
                    "failed" => crate::core::batch::BatchStatus::Failed,
                    "in_progress" => crate::core::batch::BatchStatus::InProgress,
                    "finalizing" => crate::core::batch::BatchStatus::Finalizing,
                    "completed" => crate::core::batch::BatchStatus::Completed,
                    "expired" => crate::core::batch::BatchStatus::Expired,
                    "cancelling" => crate::core::batch::BatchStatus::Cancelling,
                    "cancelled" => crate::core::batch::BatchStatus::Cancelled,
                    _ => crate::core::batch::BatchStatus::Failed,
                };

                crate::core::batch::BatchRecord {
                    id: model.id,
                    object: model.object,
                    endpoint: model.endpoint,
                    input_file_id: model.input_file_id,
                    completion_window: model.completion_window,
                    status,
                    output_file_id: model.output_file_id,
                    error_file_id: model.error_file_id,
                    created_at: model.created_at.with_timezone(&chrono::Utc),
                    in_progress_at: model.in_progress_at.map(|t| t.with_timezone(&chrono::Utc)),
                    expires_at: None, // TODO: Add expires_at field to database schema
                    finalizing_at: model.finalizing_at.map(|t| t.with_timezone(&chrono::Utc)),
                    completed_at: model.completed_at.map(|t| t.with_timezone(&chrono::Utc)),
                    failed_at: model.failed_at.map(|t| t.with_timezone(&chrono::Utc)),
                    expired_at: model.expired_at.map(|t| t.with_timezone(&chrono::Utc)),
                    cancelling_at: model.cancelling_at.map(|t| t.with_timezone(&chrono::Utc)),
                    cancelled_at: model.cancelled_at.map(|t| t.with_timezone(&chrono::Utc)),
                    request_counts: crate::core::batch::BatchRequestCounts {
                        total: model.request_counts_total.unwrap_or(0),
                        completed: model.request_counts_completed.unwrap_or(0),
                        failed: model.request_counts_failed.unwrap_or(0),
                    },
                    metadata: model.metadata.and_then(|m| serde_json::from_str(&m).ok()),
                }
            })
            .collect();

        Ok(batch_records)
    }

    /// Get batch results
    pub async fn get_batch_results(
        &self,
        _batch_id: &str,
    ) -> Result<Option<Vec<serde_json::Value>>> {
        // TODO: Implement batch results retrieval
        warn!("get_batch_results not implemented yet");
        Ok(None)
    }

    /// Get batch request
    pub async fn get_batch_request(
        &self,
        _batch_id: &str,
    ) -> Result<Option<crate::core::batch::BatchRequest>> {
        // TODO: Implement batch request retrieval
        warn!("get_batch_request not implemented yet");
        Ok(None)
    }

    /// Store batch results
    pub async fn store_batch_results(
        &self,
        _batch_id: &str,
        _results: &[serde_json::Value],
    ) -> Result<()> {
        // TODO: Implement batch results storage
        warn!("store_batch_results not implemented yet");
        Ok(())
    }

    /// Update batch progress
    pub async fn update_batch_progress(
        &self,
        batch_id: &str,
        completed: i32,
        failed: i32,
    ) -> Result<()> {
        debug!(
            "Updating batch progress: {} - completed: {}, failed: {}",
            batch_id, completed, failed
        );

        let batch_model = entities::Batch::find_by_id(batch_id)
            .one(&self.db)
            .await
            .map_err(GatewayError::Database)?
            .ok_or_else(|| GatewayError::NotFound("Batch not found".to_string()))?;

        let mut active_model: entities::batch::ActiveModel = batch_model.into();
        active_model.request_counts_completed = Set(Some(completed));
        active_model.request_counts_failed = Set(Some(failed));

        active_model
            .update(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        Ok(())
    }

    /// Mark batch as completed
    pub async fn mark_batch_completed(&self, batch_id: &str) -> Result<()> {
        debug!("Marking batch as completed: {}", batch_id);

        let batch_model = entities::Batch::find_by_id(batch_id)
            .one(&self.db)
            .await
            .map_err(GatewayError::Database)?
            .ok_or_else(|| GatewayError::NotFound("Batch not found".to_string()))?;

        let mut active_model: entities::batch::ActiveModel = batch_model.into();
        active_model.status = Set("completed".to_string());
        active_model.completed_at = Set(Some(chrono::Utc::now().into()));

        active_model
            .update(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        Ok(())
    }
}
