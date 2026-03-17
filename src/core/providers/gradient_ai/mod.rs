//! Gradient AI Provider
//!
//! Gradient AI provides AI agents and models with RAG capabilities.
//! This implementation provides access to Gradient AI's OpenAI-compatible API
//! with support for knowledge base integration and retrieval features.

mod config;
mod error;
mod provider;

pub use config::GradientAIConfig;
pub use error::{GradientAIError, GradientAIErrorMapper};
pub use provider::GradientAIProvider;
