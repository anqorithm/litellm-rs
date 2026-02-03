//! OAuth 2.0 / SSO support for enterprise authentication
//!
//! This module provides OAuth 2.0 and Single Sign-On (SSO) authentication
//! capabilities for enterprise environments.

#![allow(dead_code)]
#![allow(unused_imports)]

pub mod client;
pub mod config;
pub mod handlers;
pub mod middleware;
pub mod providers;
pub mod session;
pub mod types;

#[cfg(test)]
mod tests;

// Re-export commonly used types
pub use client::OAuthClient;
pub use config::{OAuthConfig, OAuthProvider};
pub use handlers::configure_routes;
pub use middleware::OAuthMiddleware;
pub use providers::{OidcDiscovery, OidcProvider, OidcProviderConfig};
pub use session::{InMemorySessionStore, SessionStore};
pub use types::{OAuthState, TokenResponse, UserInfo};

#[cfg(feature = "redis")]
pub use session::RedisSessionStore;
