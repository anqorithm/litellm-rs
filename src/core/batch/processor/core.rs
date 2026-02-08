//! Core BatchProcessor struct and public API methods

use super::super::types::*;
use crate::storage::database::Database;
use crate::utils::error::error::{GatewayError, Result};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

/// Batch processor for handling batch operations
pub struct BatchProcessor {
    /// Database connection
    pub(super) database: Arc<Database>,
    /// Active batches
    pub(super) active_batches: Arc<RwLock<HashMap<String, BatchResponse>>>,
    /// Batch results storage
    pub(super) results_storage: Arc<RwLock<HashMap<String, Vec<BatchResult>>>>,
}

impl BatchProcessor {
    /// Create a new batch processor
    pub fn new(database: Arc<Database>) -> Self {
        Self {
            database,
            active_batches: Arc::new(RwLock::new(HashMap::new())),
            results_storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new batch
    pub async fn create_batch(&self, request: BatchRequest) -> Result<BatchResponse> {
        info!("Creating batch: {}", request.batch_id);

        // Validate batch request
        self.validate_batch_request(&request).await?;

        let batch_response = BatchResponse {
            id: request.batch_id.clone(),
            object: "batch".to_string(),
            endpoint: self.get_endpoint_for_batch_type(&request.batch_type),
            status: BatchStatus::Validating,
            created_at: Utc::now(),
            completed_at: None,
            expires_at: Some(
                Utc::now()
                    + chrono::Duration::hours(request.completion_window.unwrap_or(24) as i64),
            ),
            input_file_id: None,
            output_file_id: None,
            error_file_id: None,
            request_counts: BatchRequestCounts {
                total: request.requests.len() as i32,
                completed: 0,
                failed: 0,
            },
            metadata: Some(
                serde_json::to_value(request.metadata.clone()).unwrap_or(serde_json::Value::Null),
            ),
            completion_window: format!("{}h", request.completion_window.unwrap_or(24)),
            in_progress_at: None,
            finalizing_at: None,
            failed_at: None,
            expired_at: None,
            cancelling_at: None,
            cancelled_at: None,
        };

        // Store batch in database
        self.database.create_batch(&request).await?;

        // Add to active batches
        {
            let mut active = self.active_batches.write().await;
            active.insert(request.batch_id.clone(), batch_response.clone());
        }

        // Start processing in background
        let processor = self.clone();
        let batch_id = request.batch_id.clone();
        tokio::spawn(async move {
            if let Err(e) = processor.process_batch(batch_id).await {
                error!("Batch processing failed: {}", e);
            }
        });

        Ok(batch_response)
    }

    /// Get batch status
    pub async fn get_batch(&self, batch_id: &str) -> Result<Option<BatchResponse>> {
        // Check active batches first
        {
            let active = self.active_batches.read().await;
            if let Some(batch) = active.get(batch_id) {
                return Ok(Some(batch.clone()));
            }
        }

        // Check database and convert BatchRequest to BatchResponse
        if let Some(batch_request) = self.database.get_batch_request(batch_id).await? {
            // Convert BatchRequest to BatchResponse
            let now = chrono::Utc::now();
            let batch_response = BatchResponse {
                id: batch_request.batch_id.clone(),
                object: "batch".to_string(),
                endpoint: "/v1/chat/completions".to_string(),
                input_file_id: Some(batch_request.batch_id.clone()),
                completion_window: "24h".to_string(),
                status: BatchStatus::Completed,
                output_file_id: Some(format!("{}_output", batch_request.batch_id)),
                error_file_id: None,
                created_at: now,
                in_progress_at: Some(now),
                expires_at: Some(now + chrono::Duration::try_days(1).unwrap_or_default()),
                finalizing_at: None,
                completed_at: Some(now),
                failed_at: None,
                expired_at: None,
                cancelling_at: None,
                cancelled_at: None,
                request_counts: BatchRequestCounts {
                    total: batch_request.requests.len() as i32,
                    completed: batch_request.requests.len() as i32,
                    failed: 0,
                },
                metadata: Some(
                    serde_json::to_value(batch_request.metadata).unwrap_or(serde_json::Value::Null),
                ),
            };
            return Ok(Some(batch_response));
        }

        Ok(None)
    }

    /// Cancel a batch
    pub async fn cancel_batch(&self, batch_id: &str) -> Result<BatchResponse> {
        info!("Cancelling batch: {}", batch_id);

        let mut batch = self
            .get_batch(batch_id)
            .await?
            .ok_or_else(|| GatewayError::NotFound("Batch not found".to_string()))?;

        // Only allow cancellation of certain statuses
        match batch.status {
            BatchStatus::Validating | BatchStatus::InProgress => {
                batch.status = BatchStatus::Cancelling;

                // Update in active batches
                {
                    let mut active = self.active_batches.write().await;
                    active.insert(batch_id.to_string(), batch.clone());
                }

                // Update in database
                self.database
                    .update_batch_status(batch_id, &format!("{:?}", batch.status))
                    .await?;

                Ok(batch)
            }
            _ => Err(GatewayError::InvalidRequest(
                "Batch cannot be cancelled in current status".to_string(),
            )),
        }
    }

    /// List batches for a user
    pub async fn list_batches(
        &self,
        _user_id: &str,
        limit: Option<u32>,
        after: Option<&str>,
    ) -> Result<Vec<BatchResponse>> {
        // Database returns BatchRecord, convert to BatchResponse
        let records = self
            .database
            .list_batches(Some(limit.unwrap_or(20) as i32), after)
            .await?;

        let responses = records
            .into_iter()
            .map(|record| BatchResponse {
                id: record.id,
                object: record.object,
                endpoint: record.endpoint,
                status: record.status,
                created_at: record.created_at,
                completed_at: record.completed_at,
                expires_at: record.expires_at,
                input_file_id: record.input_file_id,
                output_file_id: record.output_file_id,
                error_file_id: record.error_file_id,
                request_counts: record.request_counts,
                metadata: record.metadata,
                completion_window: record.completion_window,
                in_progress_at: record.in_progress_at,
                finalizing_at: record.finalizing_at,
                failed_at: record.failed_at,
                expired_at: record.expired_at,
                cancelling_at: record.cancelling_at,
                cancelled_at: record.cancelled_at,
            })
            .collect();

        Ok(responses)
    }

    /// Get batch results
    pub async fn get_batch_results(&self, batch_id: &str) -> Result<Vec<BatchResult>> {
        // Check in-memory results first
        {
            let results = self.results_storage.read().await;
            if let Some(batch_results) = results.get(batch_id) {
                return Ok(batch_results.clone());
            }
        }

        // Check database
        match self.database.get_batch_results(batch_id).await? {
            Some(json_results) => {
                // Convert JSON values to BatchResult
                let results: Vec<BatchResult> = json_results
                    .into_iter()
                    .filter_map(|v| serde_json::from_value(v).ok())
                    .collect();
                Ok(results)
            }
            None => Ok(Vec::new()),
        }
    }
}

impl Clone for BatchProcessor {
    fn clone(&self) -> Self {
        Self {
            database: self.database.clone(),
            active_batches: self.active_batches.clone(),
            results_storage: self.results_storage.clone(),
        }
    }
}
