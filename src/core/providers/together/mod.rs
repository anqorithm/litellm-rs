//! Together AI Provider
//!
//! Together AI provides access to open-source models with high-performance inference.
//! This implementation provides access to chat completions, embeddings, and rerank
//! through Together's OpenAI-compatible API.
//!
//! Docs: <https://docs.together.ai/reference>

// Core modules
mod config;
mod error;
mod model_info;
mod provider;
mod rerank;

// Feature modules
pub mod streaming;

// Tests

// Re-export main types for external use
pub use config::TogetherConfig;
pub use error::{TogetherError, TogetherErrorMapper};
pub use model_info::{TogetherModel, get_model_info, is_function_calling_model};
pub use provider::TogetherProvider;
pub use rerank::{RerankRequest, RerankResponse, RerankResult};
