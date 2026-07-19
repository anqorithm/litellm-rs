//! Custom HTTPX Provider Implementation
//!
//! A flexible provider for custom HTTP-based LLM endpoints.
//!
//! Deprecated in 0.6.0 and scheduled for removal in 0.7.0. See
//! `docs/providers/GH837-migration-0.6-to-0.7.md` for supported alternatives.

pub mod config;
pub mod error_mapper;
pub mod model_info;
pub mod provider;

#[deprecated(
    since = "0.6.0",
    note = "custom_api will be removed in 0.7.0; migrate to a registry-backed OpenAI-compatible provider or a dedicated typed provider integration"
)]
pub use config::CustomHttpxConfig;
#[deprecated(
    since = "0.6.0",
    note = "custom_api will be removed in 0.7.0; migrate to a registry-backed OpenAI-compatible provider or a dedicated typed provider integration"
)]
pub use error_mapper::CustomApiErrorMapper;
#[deprecated(
    since = "0.6.0",
    note = "custom_api will be removed in 0.7.0; migrate to a registry-backed OpenAI-compatible provider or a dedicated typed provider integration"
)]
pub use provider::CustomHttpxProvider;

pub const PROVIDER_NAME: &str = "custom_httpx";
