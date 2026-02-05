//! Fine-tuning providers
//!
//! Provider-specific implementations for fine-tuning.

pub mod openai;

use async_trait::async_trait;
use std::sync::Arc;

use super::types::{
    CreateJobRequest, FineTuningCheckpoint, FineTuningJob, ListEventsParams, ListEventsResponse,
    ListJobsParams, ListJobsResponse,
};

/// Result type for fine-tuning operations
pub type FineTuningResult<T> = Result<T, FineTuningError>;

/// Fine-tuning error types
#[derive(Debug, thiserror::Error)]
pub enum FineTuningError {
    #[error("Provider not found: {provider}")]
    ProviderNotFound { provider: String },

    #[error("Job not found: {job_id}")]
    JobNotFound { job_id: String },

    #[error("Invalid request: {message}")]
    InvalidRequest { message: String },

    #[error("Authentication error: {message}")]
    Authentication { message: String },

    #[error("Rate limited: retry after {retry_after_seconds}s")]
    RateLimited { retry_after_seconds: u64 },

    #[error("Provider error: {message}")]
    Provider { message: String },

    #[error("Network error: {message}")]
    Network { message: String },

    #[error("Fine-tuning error: {message}")]
    Other { message: String },
}

impl FineTuningError {
    pub fn provider_not_found(provider: impl Into<String>) -> Self {
        Self::ProviderNotFound {
            provider: provider.into(),
        }
    }

    pub fn job_not_found(job_id: impl Into<String>) -> Self {
        Self::JobNotFound {
            job_id: job_id.into(),
        }
    }

    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::InvalidRequest {
            message: message.into(),
        }
    }

    pub fn auth(message: impl Into<String>) -> Self {
        Self::Authentication {
            message: message.into(),
        }
    }

    pub fn provider(message: impl Into<String>) -> Self {
        Self::Provider {
            message: message.into(),
        }
    }

    pub fn network(message: impl Into<String>) -> Self {
        Self::Network {
            message: message.into(),
        }
    }

    pub fn other(message: impl Into<String>) -> Self {
        Self::Other {
            message: message.into(),
        }
    }
}

/// Fine-tuning provider trait
#[async_trait]
pub trait FineTuningProvider: Send + Sync {
    /// Get the provider name
    fn name(&self) -> &'static str;

    /// Create a new fine-tuning job
    async fn create_job(&self, request: CreateJobRequest) -> FineTuningResult<FineTuningJob>;

    /// List fine-tuning jobs
    async fn list_jobs(&self, params: ListJobsParams) -> FineTuningResult<ListJobsResponse>;

    /// Get a specific fine-tuning job
    async fn get_job(&self, job_id: &str) -> FineTuningResult<FineTuningJob>;

    /// Cancel a fine-tuning job
    async fn cancel_job(&self, job_id: &str) -> FineTuningResult<FineTuningJob>;

    /// List events for a fine-tuning job
    async fn list_events(
        &self,
        job_id: &str,
        params: ListEventsParams,
    ) -> FineTuningResult<ListEventsResponse>;

    /// List checkpoints for a fine-tuning job (optional)
    async fn list_checkpoints(&self, _job_id: &str) -> FineTuningResult<Vec<FineTuningCheckpoint>> {
        Ok(vec![])
    }
}

/// Type alias for boxed provider
pub type BoxedFineTuningProvider = Arc<dyn FineTuningProvider>;

pub use openai::OpenAIFineTuningProvider;
