//! Amazon Nova Provider
//!
//! Provider for Amazon Nova multimodal models using an OpenAI-compatible API.
//! Deprecated in 0.6.0; operational until the planned 0.7.0 catalog demotion.

pub mod config;
pub mod error;
pub mod models;
pub mod provider;

pub use config::AmazonNovaConfig;
pub use error::AmazonNovaErrorMapper;
pub use models::{AmazonNovaModel, AmazonNovaModelRegistry};
pub use provider::AmazonNovaProvider;
