//! IBM Watsonx Provider
//!
//! IBM Watsonx.ai is IBM's enterprise AI platform that provides access to
//! foundation models and tools for building AI applications. This implementation
//! provides access to Watsonx models through their REST API.
//!
//! Supported features:
//! - Chat completions
//! - Streaming chat completions
//! - Tool calling (for supported models)
//!
//! References:
//! - API Docs: <https://cloud.ibm.com/apidocs/watsonx-ai>

// Core modules
mod config;
mod error;
mod model_info;
mod provider;
mod streaming;

// Tests

// Re-export main types for external use
pub use config::WatsonxConfig;
pub use error::{WatsonxError, WatsonxErrorMapper};
pub use model_info::{WatsonxModel, get_available_models, get_model_info};
pub use provider::WatsonxProvider;
