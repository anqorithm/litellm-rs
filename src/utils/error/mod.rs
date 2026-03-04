//! Error Handling utilities
//!
//! This module provides comprehensive error handling, recovery, and error context management.

pub mod canonical;
pub mod gateway_error;
pub mod recovery;
pub mod utils;

// Re-export commonly used types and functions
pub use canonical::{CanonicalError, ErrorCode};
pub use utils::{ErrorCategory, ErrorContext, ErrorUtils};
