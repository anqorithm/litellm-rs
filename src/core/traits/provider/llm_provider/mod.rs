//! LLM Provider module
//!
//! This module provides the unified interface for all AI providers.
//!
//! `trait_definition::LLMProvider` plus `ProviderCapability` is the runtime
//! dispatch contract. `sub_traits` is exported only for deprecated library API
//! compatibility and must not be used by new router or gateway call sites.
//! The original `llm_provider.rs` has been split into smaller modules for better maintainability.

pub mod sub_traits;
pub mod trait_definition;
mod types;
