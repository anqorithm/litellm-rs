//! Voyage AI Provider
//!
//! Voyage AI is a specialized embedding provider offering state-of-the-art
//! text embeddings for semantic search, retrieval, and RAG applications.
//!
//! Reference: <https://docs.voyageai.com/reference/embeddings-api>

mod config;
mod error;
mod model_info;
mod provider;

// Re-export main types for external use
pub use config::VoyageConfig;
pub use error::VoyageError;
pub use model_info::{VoyageModel, get_model_info};
pub use provider::VoyageProvider;
