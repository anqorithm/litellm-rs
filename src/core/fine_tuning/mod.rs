//! Fine-tuning API
//!
//! This module provides a unified interface for fine-tuning LLM models across
//! different providers (OpenAI, Azure OpenAI, etc.).
//!
//! # Features
//!
//! - Create and manage fine-tuning jobs
//! - Upload and manage training datasets
//! - Monitor job progress and events
//! - List and retrieve fine-tuned models
//!
//! # Example
//!
//! ```rust,ignore
//! use litellm_rs::core::fine_tuning::{FineTuningManager, CreateJobRequest};
//!
//! let manager = FineTuningManager::new();
//! manager.register_provider("openai", openai_provider).await;
//!
//! let job = manager.create_job("openai", CreateJobRequest {
//!     model: "gpt-3.5-turbo".to_string(),
//!     training_file: "file-abc123".to_string(),
//!     ..Default::default()
//! }).await?;
//! ```

pub mod config;
pub mod manager;
pub mod providers;
pub mod types;

pub use config::FineTuningConfig;
pub use manager::FineTuningManager;
pub use types::{
    CreateJobRequest, FineTuningEvent, FineTuningJob, FineTuningJobStatus, Hyperparameters,
    ListJobsParams, ListJobsResponse,
};
