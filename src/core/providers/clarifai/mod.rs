//! Clarifai Provider
//!
//! Clarifai provides AI/ML models through their platform with an OpenAI-compatible API.
//! This implementation provides access to models hosted on Clarifai's infrastructure.

mod config;
mod error;
mod provider;

pub use config::ClarifaiConfig;
pub use error::ClarifaiError;
pub use provider::ClarifaiProvider;
