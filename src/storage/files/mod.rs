//! File storage implementation
//!
//! This module provides file storage functionality with support for local and cloud storage.

mod local;
mod s3;
mod storage;
#[cfg(test)]
mod tests;
mod types;

// Re-export public types
pub use local::LocalStorage;
pub use s3::S3Storage;
pub use types::{FileMetadata, FileStorage};
