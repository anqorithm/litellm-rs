//! HuggingFace Provider
//!
//! HuggingFace Hub integration supporting Inference Providers and Inference Endpoints.
//! This implementation provides access to ML models hosted on the HuggingFace Hub.
//!
//! ## Features
//! - Serverless Inference Providers (Together AI, Sambanova, Fireworks, etc.)
//! - Dedicated Inference Endpoints
//! - Text Generation Inference (TGI)
//! - Embeddings via text-embeddings-inference
//! - OpenAI-compatible API format

mod config;
mod embedding;
mod error;
mod models;
mod provider;

// Re-exports for easy access
pub use config::HuggingFaceConfig;
pub use error::HuggingFaceError;
pub use models::{HuggingFaceTask, InferenceProvider};
pub use provider::HuggingFaceProvider;
