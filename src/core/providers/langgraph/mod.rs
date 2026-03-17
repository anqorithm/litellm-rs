//! LangGraph Provider
//!
//! LangGraph is part of the LangChain ecosystem, providing graph-based
//! workflow orchestration for AI agents. This provider integrates with
//! LangGraph Cloud for executing graphs and managing agent state.
//!
//! # Features
//!
//! - Connect to LangGraph Cloud
//! - Execute graphs/workflows
//! - State management for multi-turn conversations
//! - Agent orchestration with checkpoints
//!
//! # Authentication
//!
//! Requires a LangSmith/LangGraph API key. Set via:
//! - `LANGGRAPH_API_KEY` environment variable
//! - `LANGSMITH_API_KEY` environment variable (fallback)
//!
//! # Example
//!
//! ```rust,ignore
//! use litellm_rs::core::providers::langgraph::{LangGraphProvider, LangGraphConfig};
//!
//! let config = LangGraphConfig::from_env();
//! let provider = LangGraphProvider::new(config)?;
//! ```

mod config;
mod error;
mod models;
mod provider;

// Re-export main types for external use
pub use config::LangGraphConfig;
pub use error::LangGraphErrorMapper;
pub use models::{GraphInfo, ThreadState, get_langgraph_models};
pub use provider::LangGraphProvider;
