//! Langfuse HTTP Client
//!
//! HTTP client for communicating with the Langfuse API.

use std::sync::Arc;
use std::time::Duration;

use reqwest::Client;
use thiserror::Error;
use tracing::{debug, error, warn};

use super::config::LangfuseConfig;
use super::types::{IngestionBatch, IngestionEvent, IngestionResponse};
use crate::utils::net::http::create_custom_client;

/// Langfuse client errors
#[derive(Debug, Error)]
pub enum LangfuseError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// Authentication error
    #[error("Authentication failed: {0}")]
    Authentication(String),

    /// API error response
    #[error("API error (status {status}): {message}")]
    ApiError { status: u16, message: String },

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Client disabled
    #[error("Langfuse client is disabled")]
    Disabled,
}

/// HTTP client timeout
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Langfuse HTTP client for API communication
#[derive(Debug, Clone)]
pub struct LangfuseClient {
    /// HTTP client
    client: Arc<Client>,
    /// Configuration
    config: LangfuseConfig,
}

impl LangfuseClient {
    /// Create a new Langfuse client
    pub fn new(config: LangfuseConfig) -> Result<Self, LangfuseError> {
        if !config.is_valid() {
            if !config.enabled {
                return Err(LangfuseError::Disabled);
            }
            return Err(LangfuseError::Configuration(
                "Missing public_key or secret_key".to_string(),
            ));
        }

        let client = create_custom_client(Duration::from_secs(DEFAULT_TIMEOUT_SECS))?;

        Ok(Self {
            client: Arc::new(client),
            config,
        })
    }

    /// Create a client from environment variables
    pub fn from_env() -> Result<Self, LangfuseError> {
        Self::new(LangfuseConfig::from_env())
    }

    /// Check if the client is in debug mode
    pub fn is_debug(&self) -> bool {
        self.config.debug
    }

    /// Get the configuration
    pub fn config(&self) -> &LangfuseConfig {
        &self.config
    }

    /// Send a batch of ingestion events
    pub async fn ingest(&self, batch: IngestionBatch) -> Result<IngestionResponse, LangfuseError> {
        if batch.is_empty() {
            return Ok(IngestionResponse {
                successes: Vec::new(),
                errors: Vec::new(),
            });
        }

        // Debug mode - log instead of sending
        if self.config.debug {
            debug!("Langfuse debug mode - would send {} events", batch.len());
            for event in &batch.batch {
                debug!("Event: {:?}", event);
            }
            return Ok(IngestionResponse {
                successes: batch
                    .batch
                    .iter()
                    .map(|e| super::types::IngestionSuccess {
                        id: e.event_id().to_string(),
                        status: 200,
                    })
                    .collect(),
                errors: Vec::new(),
            });
        }

        let url = self.config.ingestion_endpoint();
        let auth_header = self.config.auth_header().ok_or_else(|| {
            LangfuseError::Configuration("Missing authentication credentials".to_string())
        })?;

        debug!("Sending {} events to Langfuse: {}", batch.len(), url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", auth_header)
            .header("Content-Type", "application/json")
            .json(&batch)
            .send()
            .await?;

        let status = response.status();

        if status.is_success() {
            let result: IngestionResponse = response.json().await?;
            debug!(
                "Langfuse ingestion complete: {} successes, {} errors",
                result.successes.len(),
                result.errors.len()
            );
            Ok(result)
        } else if status.as_u16() == 401 || status.as_u16() == 403 {
            let message = response.text().await.unwrap_or_default();
            error!("Langfuse authentication failed: {}", message);
            Err(LangfuseError::Authentication(message))
        } else {
            let message = response.text().await.unwrap_or_default();
            warn!("Langfuse API error ({}): {}", status.as_u16(), message);
            Err(LangfuseError::ApiError {
                status: status.as_u16(),
                message,
            })
        }
    }

    /// Send a single event
    pub async fn send_event(&self, event: IngestionEvent) -> Result<(), LangfuseError> {
        let mut batch = IngestionBatch::new();
        batch.add(event);

        let response = self.ingest(batch).await?;

        if !response.errors.is_empty() {
            let error = &response.errors[0];
            return Err(LangfuseError::ApiError {
                status: error.status,
                message: error
                    .message
                    .clone()
                    .or_else(|| error.error.clone())
                    .unwrap_or_else(|| "Unknown error".to_string()),
            });
        }

        Ok(())
    }

    /// Health check - verify connection to Langfuse
    pub async fn health_check(&self) -> Result<bool, LangfuseError> {
        if !self.config.is_valid() {
            return Ok(false);
        }

        // Send empty batch to verify credentials
        let batch = IngestionBatch::new();
        match self.ingest(batch).await {
            Ok(_) => Ok(true),
            Err(LangfuseError::Authentication(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }
}

/// Async batch sender for background processing
pub struct BatchSender {
    client: LangfuseClient,
    batch: parking_lot::Mutex<IngestionBatch>,
    batch_size: usize,
}

impl BatchSender {
    /// Create a new batch sender
    pub fn new(client: LangfuseClient) -> Self {
        let batch_size = client.config.batch_size;
        Self {
            client,
            batch: parking_lot::Mutex::new(IngestionBatch::new()),
            batch_size,
        }
    }

    /// Add an event to the batch, returns true if batch should be flushed
    pub fn add(&self, event: IngestionEvent) -> bool {
        let mut batch = self.batch.lock();
        batch.add(event);
        batch.len() >= self.batch_size
    }

    /// Flush the current batch
    pub async fn flush(&self) -> Result<IngestionResponse, LangfuseError> {
        let batch = {
            let mut batch = self.batch.lock();
            if batch.is_empty() {
                return Ok(IngestionResponse {
                    successes: Vec::new(),
                    errors: Vec::new(),
                });
            }
            std::mem::take(&mut *batch)
        };

        self.client.ingest(batch).await
    }

    /// Get current batch size
    pub fn pending_count(&self) -> usize {
        self.batch.lock().len()
    }

    /// Check if client is in debug mode
    pub fn is_debug(&self) -> bool {
        self.client.is_debug()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::integrations::langfuse::types::{Generation, Trace};

    fn test_config() -> LangfuseConfig {
        LangfuseConfig {
            public_key: Some("pk-test".to_string()),
            secret_key: Some("sk-test".to_string()),
            host: "https://cloud.langfuse.com".to_string(),
            enabled: true,
            batch_size: 10,
            flush_interval_ms: 1000,
            debug: true, // Use debug mode for tests
            release: None,
        }
    }

    #[test]
    fn test_client_creation() {
        let config = test_config();
        let client = LangfuseClient::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_disabled() {
        let mut config = test_config();
        config.enabled = false;
        let client = LangfuseClient::new(config);
        assert!(matches!(client, Err(LangfuseError::Disabled)));
    }

    #[test]
    fn test_client_missing_credentials() {
        let config = LangfuseConfig::default();
        let client = LangfuseClient::new(config);
        assert!(matches!(client, Err(LangfuseError::Configuration(_))));
    }

    #[test]
    fn test_client_debug_mode() {
        let config = test_config();
        let client = LangfuseClient::new(config).unwrap();
        assert!(client.is_debug());
    }

    #[tokio::test]
    async fn test_ingest_empty_batch() {
        let config = test_config();
        let client = LangfuseClient::new(config).unwrap();

        let batch = IngestionBatch::new();
        let result = client.ingest(batch).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.successes.is_empty());
        assert!(response.errors.is_empty());
    }

    #[tokio::test]
    async fn test_ingest_debug_mode() {
        let config = test_config();
        let client = LangfuseClient::new(config).unwrap();

        let mut batch = IngestionBatch::new();
        batch.add(IngestionEvent::trace_create(Trace::new().name("test")));

        let result = client.ingest(batch).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.successes.len(), 1);
        assert!(response.errors.is_empty());
    }

    #[test]
    fn test_batch_sender_creation() {
        let config = test_config();
        let client = LangfuseClient::new(config).unwrap();
        let sender = BatchSender::new(client);

        assert_eq!(sender.pending_count(), 0);
    }

    #[test]
    fn test_batch_sender_add() {
        let config = test_config();
        let client = LangfuseClient::new(config).unwrap();
        let sender = BatchSender::new(client);

        let trace = Trace::new().name("test");
        let should_flush = sender.add(IngestionEvent::trace_create(trace));

        assert!(!should_flush);
        assert_eq!(sender.pending_count(), 1);
    }

    #[test]
    fn test_batch_sender_triggers_flush() {
        let mut config = test_config();
        config.batch_size = 2;
        let client = LangfuseClient::new(config).unwrap();
        let sender = BatchSender::new(client);

        sender.add(IngestionEvent::trace_create(Trace::new()));
        let should_flush = sender.add(IngestionEvent::trace_create(Trace::new()));

        assert!(should_flush);
        assert_eq!(sender.pending_count(), 2);
    }

    #[tokio::test]
    async fn test_batch_sender_flush() {
        let config = test_config();
        let client = LangfuseClient::new(config).unwrap();
        let sender = BatchSender::new(client);

        sender.add(IngestionEvent::trace_create(Trace::new().name("test")));
        sender.add(IngestionEvent::generation_create(
            Generation::new("trace-id").model("gpt-4"),
        ));

        let result = sender.flush().await;
        assert!(result.is_ok());
        assert_eq!(sender.pending_count(), 0);
    }

    #[tokio::test]
    async fn test_batch_sender_flush_empty() {
        let config = test_config();
        let client = LangfuseClient::new(config).unwrap();
        let sender = BatchSender::new(client);

        let result = sender.flush().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_display() {
        let error = LangfuseError::Configuration("test error".to_string());
        assert_eq!(error.to_string(), "Configuration error: test error");

        let error = LangfuseError::ApiError {
            status: 500,
            message: "Server error".to_string(),
        };
        assert_eq!(error.to_string(), "API error (status 500): Server error");
    }
}
