//! Legacy cache manager compatibility module.
//!
//! Canonical cache boundaries:
//! - `crate::core::cache`: key-based deterministic cache (DualCache/LLMCache).
//! - `crate::core::semantic_cache`: vector-similarity semantic cache.
//!
//! New code should prefer `core::cache` and `core::semantic_cache`.

pub mod manager;
pub mod types;

#[cfg(test)]
mod tests;
