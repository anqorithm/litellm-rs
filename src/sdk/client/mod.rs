//! LLM Client module
//!
//! This module provides a full-featured LLM client with support for multiple providers,
//! intelligent routing, load balancing, and comprehensive statistics tracking.

mod llm_client;
mod completions;
mod embeddings;
mod routing;
mod stats;
mod types;

#[cfg(test)]
mod tests;

// Re-export public types and the main client
pub use llm_client::LLMClient;
pub use types::{LoadBalancer, LoadBalancingStrategy, ProviderStats};
