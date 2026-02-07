//! Common test utilities for litellm-rs
//!
//! This module provides shared test infrastructure for all tests:
//! - In-memory SQLite database support
//! - Test fixtures and data factories
//! - Provider test utilities
//! - Custom assertions and helpers
//!
//! # Usage
//!
//! ```rust
//! use crate::common::{database, fixtures, providers};
//!
//! #[tokio::test]
//! async fn my_test() {
//!     let db = database::TestDatabase::new().await;
//!     let user = fixtures::UserFactory::create();
//!     // ...
//! }
//! ```

pub mod assertions;
#[cfg(feature = "storage")]
pub mod database;
pub mod fixtures;
pub mod providers;

// Re-export commonly used items
#[cfg(feature = "storage")]
pub use database::TestDatabase;
pub use fixtures::{ChatRequestFactory, UserFactory};

/// Skip test if environment variable is not set
#[macro_export]
macro_rules! skip_without_env {
    ($var:expr) => {
        if std::env::var($var).is_err() {
            eprintln!("Skipping test: {} environment variable not set", $var);
            return;
        }
    };
}

/// Skip test if API key is not available
#[macro_export]
macro_rules! skip_without_api_key {
    ($provider:expr) => {
        let key_var = match $provider {
            "openai" => "OPENAI_API_KEY",
            "anthropic" => "ANTHROPIC_API_KEY",
            "groq" => "GROQ_API_KEY",
            "gemini" | "google" => "GOOGLE_API_KEY",
            "azure" => "AZURE_OPENAI_API_KEY",
            "cohere" => "COHERE_API_KEY",
            "mistral" => "MISTRAL_API_KEY",
            _ => {
                panic!("Unknown provider: {}", $provider);
            }
        };
        if std::env::var(key_var).is_err() {
            eprintln!(
                "Skipping test: {} not set for {} provider",
                key_var, $provider
            );
            return;
        }
    };
}

/// Assert that a result is Ok and return the value
#[macro_export]
macro_rules! assert_ok {
    ($expr:expr) => {
        match $expr {
            Ok(v) => v,
            Err(e) => panic!("Expected Ok, got Err: {:?}", e),
        }
    };
}

/// Assert that a result is Err
#[macro_export]
macro_rules! assert_err {
    ($expr:expr) => {
        match $expr {
            Ok(v) => panic!("Expected Err, got Ok: {:?}", v),
            Err(e) => e,
        }
    };
}
