//! Baseten Provider
//!
//! Baseten provides serverless inference for machine learning models.
//! This implementation provides access to Baseten's OpenAI-compatible API
//! for both Model API and dedicated deployments.

mod config;
mod error;
mod provider;

pub use config::BasetenConfig;
pub use error::BasetenError;
pub use provider::BasetenProvider;
