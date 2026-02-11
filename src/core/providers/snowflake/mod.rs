//! Snowflake Cortex AI Provider
//!
//! Snowflake Cortex provides access to LLM capabilities directly within Snowflake.
//! This implementation provides access to Cortex LLM functions through their REST API.
//!
//! Supported features:
//! - Chat completions
//! - Streaming chat completions
//! - Tool calling (for supported models like Claude 3.5 Sonnet)
//!
//! References:
//! - API Docs: <https://docs.snowflake.com/en/user-guide/snowflake-cortex/cortex-llm-rest-api>

// Core modules
mod config;
mod error;
mod model_info;
mod provider;

// Re-export main types for external use
pub use config::SnowflakeConfig;
pub use error::SnowflakeError;
pub use model_info::{SnowflakeModel, get_available_models, get_model_info};
pub use provider::SnowflakeProvider;
