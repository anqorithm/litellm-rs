//! Error Handling utilities
//!
//! This module provides comprehensive error handling, recovery, and error context management.

#[path = "error/mod.rs"]
pub mod gateway;
pub use gateway as error;
pub mod recovery;
pub mod utils;

// Re-export commonly used types and functions
pub use utils::{ErrorCategory, ErrorContext, ErrorUtils};
