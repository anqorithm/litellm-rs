//! Predibase Provider
//!
//! Predibase provides fine-tuned LLM serving.
//! API Reference: <https://docs.predibase.com/>

mod config;
mod error;
mod model_info;
mod provider;

pub use config::PredibaseConfig;
pub use error::PredibaseError;
pub use model_info::{get_available_models, get_model_info};
pub use provider::PredibaseProvider;
