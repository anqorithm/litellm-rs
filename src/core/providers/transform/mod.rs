//! Request/Response Transformation Engine
//!
//! Split into sub-modules for maintainability:
//! - `types`: Type definitions, traits, and pipeline structures
//! - `engine`: DefaultTransformEngine and concrete Transform implementations

mod engine;
mod types;

// Re-export everything at the same path as before
pub use engine::*;
pub use types::*;

// Include tests that span both modules
#[cfg(test)]
#[path = "tests_module.rs"]
mod tests_include;
