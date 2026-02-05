//! Fine-tuning types
//!
//! Data structures for fine-tuning jobs, events, and requests.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Fine-tuning job status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FineTuningJobStatus {
    /// Job is queued and waiting to start
    Queued,
    /// Job is validating files
    ValidatingFiles,
    /// Job is currently running
    Running,
    /// Job completed successfully
    Succeeded,
    /// Job failed
    Failed,
    /// Job was cancelled
    Cancelled,
}

impl std::fmt::Display for FineTuningJobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Queued => write!(f, "queued"),
            Self::ValidatingFiles => write!(f, "validating_files"),
            Self::Running => write!(f, "running"),
            Self::Succeeded => write!(f, "succeeded"),
            Self::Failed => write!(f, "failed"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Hyperparameters for fine-tuning
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Hyperparameters {
    /// Number of epochs to train for
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n_epochs: Option<u32>,

    /// Batch size for training
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_size: Option<u32>,

    /// Learning rate multiplier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub learning_rate_multiplier: Option<f64>,
}

impl Hyperparameters {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn n_epochs(mut self, n: u32) -> Self {
        self.n_epochs = Some(n);
        self
    }

    pub fn batch_size(mut self, size: u32) -> Self {
        self.batch_size = Some(size);
        self
    }

    pub fn learning_rate_multiplier(mut self, multiplier: f64) -> Self {
        self.learning_rate_multiplier = Some(multiplier);
        self
    }
}

/// Request to create a fine-tuning job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateJobRequest {
    /// Base model to fine-tune
    pub model: String,

    /// Training file ID
    pub training_file: String,

    /// Validation file ID (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_file: Option<String>,

    /// Hyperparameters for training
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hyperparameters: Option<Hyperparameters>,

    /// Suffix for the fine-tuned model name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,

    /// Seed for reproducibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,

    /// Custom metadata
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,
}

impl CreateJobRequest {
    pub fn new(model: impl Into<String>, training_file: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            training_file: training_file.into(),
            validation_file: None,
            hyperparameters: None,
            suffix: None,
            seed: None,
            metadata: HashMap::new(),
        }
    }

    pub fn validation_file(mut self, file: impl Into<String>) -> Self {
        self.validation_file = Some(file.into());
        self
    }

    pub fn hyperparameters(mut self, params: Hyperparameters) -> Self {
        self.hyperparameters = Some(params);
        self
    }

    pub fn suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = Some(suffix.into());
        self
    }

    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Fine-tuning job error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FineTuningError {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Parameter that caused the error (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,
}

/// Fine-tuning job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FineTuningJob {
    /// Job ID
    pub id: String,

    /// Object type (always "fine_tuning.job")
    #[serde(default = "default_object_type")]
    pub object: String,

    /// Base model being fine-tuned
    pub model: String,

    /// Fine-tuned model name (available after completion)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fine_tuned_model: Option<String>,

    /// Organization ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,

    /// Job status
    pub status: FineTuningJobStatus,

    /// Training file ID
    pub training_file: String,

    /// Validation file ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_file: Option<String>,

    /// Result files (model weights, etc.)
    #[serde(default)]
    pub result_files: Vec<String>,

    /// Hyperparameters used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hyperparameters: Option<Hyperparameters>,

    /// Number of trained tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trained_tokens: Option<u64>,

    /// Error information (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<FineTuningError>,

    /// Creation timestamp (Unix seconds)
    pub created_at: i64,

    /// Start timestamp (Unix seconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<i64>,

    /// Completion timestamp (Unix seconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<i64>,

    /// Estimated completion timestamp (Unix seconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_finish: Option<i64>,

    /// Model suffix
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,

    /// Seed used for training
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,

    /// Custom metadata
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,

    /// Provider name (litellm-rs specific)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}

fn default_object_type() -> String {
    "fine_tuning.job".to_string()
}

impl FineTuningJob {
    /// Check if the job is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            FineTuningJobStatus::Succeeded
                | FineTuningJobStatus::Failed
                | FineTuningJobStatus::Cancelled
        )
    }

    /// Check if the job is still running
    pub fn is_running(&self) -> bool {
        matches!(
            self.status,
            FineTuningJobStatus::Queued
                | FineTuningJobStatus::ValidatingFiles
                | FineTuningJobStatus::Running
        )
    }

    /// Get the duration in seconds (if completed)
    pub fn duration_seconds(&self) -> Option<i64> {
        match (self.started_at, self.finished_at) {
            (Some(start), Some(end)) => Some(end - start),
            _ => None,
        }
    }
}

/// Fine-tuning event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FineTuningEvent {
    /// Event ID
    pub id: String,

    /// Object type (always "fine_tuning.job.event")
    #[serde(default = "default_event_object_type")]
    pub object: String,

    /// Event type/level
    pub level: FineTuningEventLevel,

    /// Event message
    pub message: String,

    /// Event timestamp (Unix seconds)
    pub created_at: i64,

    /// Event type (e.g., "message", "metrics")
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub event_type: Option<String>,

    /// Event data (for metrics events)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<FineTuningEventData>,
}

fn default_event_object_type() -> String {
    "fine_tuning.job.event".to_string()
}

/// Fine-tuning event level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FineTuningEventLevel {
    Info,
    Warning,
    Error,
}

/// Fine-tuning event data (for metrics)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FineTuningEventData {
    /// Training step
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<u32>,

    /// Training loss
    #[serde(skip_serializing_if = "Option::is_none")]
    pub train_loss: Option<f64>,

    /// Validation loss
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_loss: Option<f64>,

    /// Training mean token accuracy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub train_mean_token_accuracy: Option<f64>,

    /// Validation mean token accuracy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_mean_token_accuracy: Option<f64>,

    /// Full validation loss
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_valid_loss: Option<f64>,

    /// Full validation mean token accuracy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_valid_mean_token_accuracy: Option<f64>,
}

/// Parameters for listing fine-tuning jobs
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListJobsParams {
    /// Cursor for pagination (job ID to start after)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,

    /// Maximum number of jobs to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

impl ListJobsParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn after(mut self, after: impl Into<String>) -> Self {
        self.after = Some(after.into());
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// Response for listing fine-tuning jobs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListJobsResponse {
    /// Object type (always "list")
    pub object: String,

    /// List of jobs
    pub data: Vec<FineTuningJob>,

    /// Whether there are more results
    pub has_more: bool,
}

/// Parameters for listing fine-tuning events
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListEventsParams {
    /// Cursor for pagination (event ID to start after)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,

    /// Maximum number of events to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

impl ListEventsParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn after(mut self, after: impl Into<String>) -> Self {
        self.after = Some(after.into());
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// Response for listing fine-tuning events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListEventsResponse {
    /// Object type (always "list")
    pub object: String,

    /// List of events
    pub data: Vec<FineTuningEvent>,

    /// Whether there are more results
    pub has_more: bool,
}

/// Fine-tuning checkpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FineTuningCheckpoint {
    /// Checkpoint ID
    pub id: String,

    /// Object type
    pub object: String,

    /// Fine-tuning job ID
    pub fine_tuning_job_id: String,

    /// Step number
    pub step_number: u32,

    /// Checkpoint model name
    pub fine_tuned_model_checkpoint: String,

    /// Metrics at this checkpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<CheckpointMetrics>,

    /// Creation timestamp
    pub created_at: i64,
}

/// Metrics for a checkpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointMetrics {
    /// Training loss
    #[serde(skip_serializing_if = "Option::is_none")]
    pub train_loss: Option<f64>,

    /// Validation loss
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_loss: Option<f64>,

    /// Training mean token accuracy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub train_mean_token_accuracy: Option<f64>,

    /// Validation mean token accuracy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_mean_token_accuracy: Option<f64>,

    /// Step number
    pub step: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_status_display() {
        assert_eq!(FineTuningJobStatus::Queued.to_string(), "queued");
        assert_eq!(FineTuningJobStatus::Running.to_string(), "running");
        assert_eq!(FineTuningJobStatus::Succeeded.to_string(), "succeeded");
        assert_eq!(FineTuningJobStatus::Failed.to_string(), "failed");
    }

    #[test]
    fn test_hyperparameters_builder() {
        let params = Hyperparameters::new()
            .n_epochs(3)
            .batch_size(4)
            .learning_rate_multiplier(0.1);

        assert_eq!(params.n_epochs, Some(3));
        assert_eq!(params.batch_size, Some(4));
        assert_eq!(params.learning_rate_multiplier, Some(0.1));
    }

    #[test]
    fn test_create_job_request_builder() {
        let request = CreateJobRequest::new("gpt-3.5-turbo", "file-abc123")
            .validation_file("file-def456")
            .suffix("my-model")
            .seed(42)
            .metadata("team", "ml")
            .hyperparameters(Hyperparameters::new().n_epochs(3));

        assert_eq!(request.model, "gpt-3.5-turbo");
        assert_eq!(request.training_file, "file-abc123");
        assert_eq!(request.validation_file, Some("file-def456".to_string()));
        assert_eq!(request.suffix, Some("my-model".to_string()));
        assert_eq!(request.seed, Some(42));
        assert_eq!(request.metadata.get("team"), Some(&"ml".to_string()));
        assert!(request.hyperparameters.is_some());
    }

    #[test]
    fn test_job_is_terminal() {
        let mut job = FineTuningJob {
            id: "ftjob-123".to_string(),
            object: "fine_tuning.job".to_string(),
            model: "gpt-3.5-turbo".to_string(),
            fine_tuned_model: None,
            organization_id: None,
            status: FineTuningJobStatus::Running,
            training_file: "file-abc".to_string(),
            validation_file: None,
            result_files: vec![],
            hyperparameters: None,
            trained_tokens: None,
            error: None,
            created_at: 0,
            started_at: None,
            finished_at: None,
            estimated_finish: None,
            suffix: None,
            seed: None,
            metadata: HashMap::new(),
            provider: None,
        };

        assert!(!job.is_terminal());
        assert!(job.is_running());

        job.status = FineTuningJobStatus::Succeeded;
        assert!(job.is_terminal());
        assert!(!job.is_running());

        job.status = FineTuningJobStatus::Failed;
        assert!(job.is_terminal());

        job.status = FineTuningJobStatus::Cancelled;
        assert!(job.is_terminal());
    }

    #[test]
    fn test_job_duration() {
        let job = FineTuningJob {
            id: "ftjob-123".to_string(),
            object: "fine_tuning.job".to_string(),
            model: "gpt-3.5-turbo".to_string(),
            fine_tuned_model: None,
            organization_id: None,
            status: FineTuningJobStatus::Succeeded,
            training_file: "file-abc".to_string(),
            validation_file: None,
            result_files: vec![],
            hyperparameters: None,
            trained_tokens: None,
            error: None,
            created_at: 1000,
            started_at: Some(1100),
            finished_at: Some(1500),
            estimated_finish: None,
            suffix: None,
            seed: None,
            metadata: HashMap::new(),
            provider: None,
        };

        assert_eq!(job.duration_seconds(), Some(400));
    }

    #[test]
    fn test_list_jobs_params_builder() {
        let params = ListJobsParams::new().after("ftjob-abc").limit(10);

        assert_eq!(params.after, Some("ftjob-abc".to_string()));
        assert_eq!(params.limit, Some(10));
    }

    #[test]
    fn test_serialization() {
        let request = CreateJobRequest::new("gpt-3.5-turbo", "file-abc123").suffix("test");

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("gpt-3.5-turbo"));
        assert!(json.contains("file-abc123"));

        let parsed: CreateJobRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.model, request.model);
    }
}
