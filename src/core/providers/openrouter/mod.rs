//! OpenRouter Provider - Unified Architecture Implementation
//!
//! OpenRouter is a unified API that provides access to multiple LLM models.
//! It's OpenAI API compatible but supports additional parameters and functionality.
//!
//! Documentation: <https://openrouter.ai/docs>

pub mod config;
pub mod models;
pub mod provider;
pub mod streaming;

pub mod client;
pub mod error;
pub mod transformer;

// Re-exports for the new architecture
pub use config::OpenRouterConfig;
pub use models::{OpenRouterModelRegistry, OpenRouterModelSpec, get_openrouter_registry};
pub use provider::OpenRouterProvider;

// No legacy top-level re-exports: use explicit module paths from call sites.
