//! SSO Provider implementations
//!
//! This module provides dedicated implementations for various SSO providers
//! with provider-specific features and OIDC Discovery support.

pub mod generic_oidc;

pub use generic_oidc::{OidcDiscovery, OidcProvider, OidcProviderConfig};
