//! Core type definition module
//!
//! Contains all core data structures and type definitions

// Split from common.rs (new modules)
pub mod cache;
pub mod context;
pub mod health;
pub mod metrics;
pub mod model;
pub mod pagination;
pub mod service;

// Split from requests.rs (new modules)
pub mod anthropic;
pub mod chat;
pub mod content;
pub mod embedding;
pub mod image;
pub mod message;
pub mod thinking;
pub mod tools;

// Original files kept as-is
pub mod config;
pub mod errors;
pub mod responses;

// Re-export all public types from split modules
pub use cache::*;
pub use context::*;
pub use health::*;
pub use metrics::*;
pub use model::*;
pub use pagination::*;
pub use service::*;

pub use anthropic::*;
pub use chat::*;
pub use content::*;
pub use embedding::*;
pub use image::*;
pub use message::*;
pub use thinking::*;
pub use tools::*;

// Re-export from remaining original files
