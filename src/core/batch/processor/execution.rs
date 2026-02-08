//! Batch execution and processing logic

use super::super::types::*;
use super::core::BatchProcessor;
use crate::core::models::openai::{ChatCompletionRequest, EmbeddingRequest};
use crate::utils::error::error::{GatewayError, Result};
use chrono::Utc;
use std::collections::HashMap;
use tracing::{debug, info};
use uuid::Uuid;

impl BatchProcessor {
    /// Process a batch
    pub(super) async fn process_batch(&self, batch_id: String) -> Result<()> {
        info!("Processing batch: {}", batch_id);

        // Update status to in progress
        self.update_batch_status(&batch_id, BatchStatus::InProgress)
            .await?;

        // Get batch request from database
        let batch_request = self
            .database
            .get_batch_request(&batch_id)
            .await?
            .ok_or_else(|| GatewayError::NotFound("Batch request not found".to_string()))?;

        // Pre-allocate results vector based on request count
        let mut results = Vec::with_capacity(batch_request.requests.len());
        let mut completed = 0;
        let mut failed = 0;

        // Process each request
        for item in &batch_request.requests {
            // Check if batch was cancelled
            if self.is_batch_cancelled(&batch_id).await? {
                break;
            }

            match self
                .process_batch_item(item, &batch_request.batch_type)
                .await
            {
                Ok(result) => {
                    results.push(result);
                    completed += 1;
                }
                Err(e) => {
                    let error_result = BatchResult {
                        custom_id: item.custom_id.clone(),
                        response: None,
                        error: Some(BatchError {
                            code: "processing_error".to_string(),
                            message: e.to_string(),
                            details: None,
                        }),
                    };
                    results.push(error_result);
                    failed += 1;
                }
            }

            // Update progress periodically
            if (completed + failed) % 100 == 0 {
                self.update_batch_progress(&batch_id, completed, failed)
                    .await?;
            }
        }

        // Store results
        {
            let mut storage = self.results_storage.write().await;
            storage.insert(batch_id.clone(), results.clone());
        }

        // Store results in database
        let json_results: Vec<serde_json::Value> = results
            .iter()
            .map(|r| serde_json::to_value(r).unwrap_or_default())
            .collect();
        self.database
            .store_batch_results(&batch_id, &json_results)
            .await?;

        // Update final status
        let final_status = if self.is_batch_cancelled(&batch_id).await? {
            BatchStatus::Cancelled
        } else {
            BatchStatus::Completed
        };

        self.update_batch_status(&batch_id, final_status).await?;
        self.update_batch_progress(&batch_id, completed, failed)
            .await?;

        // Mark completion time
        self.mark_batch_completed(&batch_id).await?;

        info!(
            "Batch processing completed: {} (completed: {}, failed: {})",
            batch_id, completed, failed
        );

        Ok(())
    }

    /// Process individual batch item
    pub(super) async fn process_batch_item(
        &self,
        item: &BatchItem,
        batch_type: &BatchType,
    ) -> Result<BatchResult> {
        debug!("Processing batch item: {}", item.custom_id);

        match batch_type {
            BatchType::ChatCompletion => {
                let request: ChatCompletionRequest = serde_json::from_value(item.body.clone())
                    .map_err(|e| {
                        GatewayError::InvalidRequest(format!("Invalid request body: {}", e))
                    })?;

                // This would need to be integrated with the actual provider system
                // For now, return a mock response
                let response = BatchHttpResponse {
                    status_code: 200,
                    headers: HashMap::new(),
                    body: serde_json::json!({
                        "id": format!("chatcmpl-batch-{}", Uuid::new_v4()),
                        "object": "chat.completion",
                        "created": Utc::now().timestamp(),
                        "model": request.model,
                        "choices": [{
                            "index": 0,
                            "message": {
                                "role": "assistant",
                                "content": "This is a batch processed response."
                            },
                            "finish_reason": "stop"
                        }],
                        "usage": {
                            "prompt_tokens": 10,
                            "completion_tokens": 8,
                            "total_tokens": 18
                        }
                    }),
                };

                Ok(BatchResult {
                    custom_id: item.custom_id.clone(),
                    response: Some(response),
                    error: None,
                })
            }
            BatchType::Embedding => {
                let request: EmbeddingRequest =
                    serde_json::from_value(item.body.clone()).map_err(|e| {
                        GatewayError::InvalidRequest(format!("Invalid request body: {}", e))
                    })?;

                let response = BatchHttpResponse {
                    status_code: 200,
                    headers: HashMap::new(),
                    body: serde_json::json!({
                        "object": "list",
                        "data": [{
                            "object": "embedding",
                            "embedding": vec![0.1; 1536], // Mock embedding
                            "index": 0
                        }],
                        "model": request.model,
                        "usage": {
                            "prompt_tokens": 5,
                            "total_tokens": 5
                        }
                    }),
                };

                Ok(BatchResult {
                    custom_id: item.custom_id.clone(),
                    response: Some(response),
                    error: None,
                })
            }
            _ => Err(GatewayError::InvalidRequest(
                "Unsupported batch type".to_string(),
            )),
        }
    }

    /// Get endpoint for batch type
    pub(super) fn get_endpoint_for_batch_type(&self, batch_type: &BatchType) -> String {
        match batch_type {
            BatchType::ChatCompletion => "/v1/chat/completions".to_string(),
            BatchType::Embedding => "/v1/embeddings".to_string(),
            BatchType::ImageGeneration => "/v1/images/generations".to_string(),
            BatchType::Custom(endpoint) => endpoint.clone(),
        }
    }
}
