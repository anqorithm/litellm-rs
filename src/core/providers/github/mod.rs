//! GitHub Models Provider
//!
//! GitHub Models provides access to various AI models through GitHub's inference API.
//! The API is OpenAI-compatible, making integration straightforward.
//!
//! This implementation follows the Python LiteLLM library pattern for GitHub Models.

mod config;
mod error;
mod model_info;
mod provider;

// Re-export main types for external use
pub use config::GitHubConfig;
pub use error::GitHubError;
pub use model_info::{GitHubModel, get_available_models, get_model_info};
pub use provider::GitHubProvider;
