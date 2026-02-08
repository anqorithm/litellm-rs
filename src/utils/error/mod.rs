//! Error Handling utilities
//!
//! This module provides comprehensive error handling, recovery, and error context management.

pub mod error;
pub mod recovery;
pub mod utils;

// Re-export commonly used types and functions
pub use utils::{ErrorCategory, ErrorContext, ErrorUtils};
