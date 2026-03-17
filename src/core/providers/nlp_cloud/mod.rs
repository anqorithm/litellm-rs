//! NLP Cloud Provider
//!
//! NLP Cloud provides multilingual NLP models via API.
//! API Reference: <https://docs.nlpcloud.com/>

mod config;
mod error;
mod model_info;
mod provider;

pub use config::NlpCloudConfig;
pub use error::NlpCloudError;
pub use model_info::{get_available_models, get_model_info};
pub use provider::NlpCloudProvider;
