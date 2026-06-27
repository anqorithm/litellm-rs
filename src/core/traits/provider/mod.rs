//! Core LLM Provider trait definitions
//!
//! Defines the implementation interfaces for providers wired into this crate.
//!
//! # Module Organization
//!
//! This module is split into three main components:
//! - `llm_provider` - The main LLMProvider trait definition
//! - `config` - ProviderConfig trait for configuration management
//! - `handle` - legacy ProviderHandle metadata wrapper
//!
//! # Design Principles
//!
//! 1. **Request uniformity**: All providers use the same request/response format
//! 2. **Capability driven**: Declare supported features through capabilities()
//! 3. **Provider agnostic**: Users don't need to know provider-specific details
//! 4. **Type safety**: Use associated types to ensure compile-time type safety
//! 5. **Async first**: All I/O operations are asynchronous
//! 6. **Observability**: Built-in cost calculation, latency statistics, and monitoring

// Module declarations
mod config;
mod handle;
pub mod llm_provider;

// Re-export all public types
pub use config::ProviderConfig;
pub use handle::ProviderHandle;
pub use llm_provider::trait_definition::LLMProvider;
