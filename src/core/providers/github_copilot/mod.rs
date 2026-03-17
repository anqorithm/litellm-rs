//! GitHub Copilot Provider
//!
//! GitHub Copilot provides AI-powered code completion and chat capabilities.
//! This provider implements the OAuth Device Flow authentication and supports
//! the Copilot Chat API which is OpenAI-compatible.
//!
//! Key features:
//! - OAuth Device Flow authentication with token caching
//! - OpenAI-compatible chat completions API
//! - Vision request support with special headers
//! - X-Initiator header for agent/user context

mod authenticator;
mod config;
mod error;
mod model_info;
mod provider;

// Re-export main types for external use
pub use authenticator::CopilotAuthenticator;
pub use config::GitHubCopilotConfig;
pub use error::GitHubCopilotError;
pub use model_info::{GitHubCopilotModel, get_available_models, get_model_info};
pub use provider::GitHubCopilotProvider;
