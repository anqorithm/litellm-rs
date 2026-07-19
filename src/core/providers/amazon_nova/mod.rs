//! Amazon Nova Provider
//!
//! Provider for Amazon Nova multimodal models using an OpenAI-compatible API.
//!
//! The native public module is deprecated in 0.6.0 and remains fully
//! operational until the planned 0.7.0 catalog demotion. See
//! `docs/providers/GH837-migration-0.6-to-0.7.md`.

pub mod config;
pub mod error;
pub mod models;
pub mod provider;

pub use config::AmazonNovaConfig;
pub use error::AmazonNovaErrorMapper;
pub use models::{AmazonNovaModel, AmazonNovaModelRegistry};
pub use provider::AmazonNovaProvider;
