//! GitHub Models Provider
//!
//! GitHub Models provides access to various AI models through GitHub's inference API.
//! The API is OpenAI-compatible, making integration straightforward.
//!
//! This implementation follows the Python LiteLLM library pattern for GitHub Models.
//!
//! Deprecated in 0.6.0; operational until the planned 0.7.0 catalog demotion.
//! The catalog route at `GITHUB_MODELS_API_BASE`
//! (<https://models.inference.ai.azure.com>) is the supported path; see
//! `registry::github_policy` for the authoritative model/pricing/capability
//! metadata.

// 0.6.0 deprecation signal for direct downstream imports. The inner attribute
// is used here because `src/core/providers/mod.rs` is at the U-16 800-line
// hard ceiling and cannot carry the outer `#[cfg_attr(not(test), deprecated)]`
// attribute used by amazon_nova/custom_api without a structural split.
#![cfg_attr(
    not(test),
    deprecated(since = "0.6.0", note = "use catalog github before 0.7")
)]

mod config;
mod error;
mod model_info;
mod provider;

#[cfg(test)]
mod tests;

// Re-export main types for external use
pub use config::GitHubConfig;
pub use error::GitHubError;
pub use model_info::{GitHubModel, get_available_models, get_model_info};
pub use provider::GitHubProvider;
