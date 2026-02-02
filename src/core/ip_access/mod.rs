//! IP Access Control System
//!
//! A decoupled IP-based access control system for the gateway.
//!
//! # Features
//!
//! - **IP Allowlist**: Only allow requests from specific IPs
//! - **IP Blocklist**: Block requests from specific IPs
//! - **CIDR Support**: Support for IP ranges using CIDR notation
//! - **Middleware**: Actix-web middleware for request filtering
//! - **Dynamic Updates**: Update rules at runtime
//!
//! # Architecture
//!
//! The IP access control system follows these principles:
//! - **Decoupled**: Independent of other security systems
//! - **Performant**: Efficient IP matching using prefix trees
//! - **Flexible**: Support for both allowlist and blocklist modes
//! - **Configurable**: Fine-grained control over behavior
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use litellm_rs::core::ip_access::{IpAccessControl, IpAccessConfig, IpAccessMode};
//!
//! let config = IpAccessConfig::default()
//!     .with_mode(IpAccessMode::Allowlist)
//!     .allow_ip("192.168.1.0/24")
//!     .allow_ip("10.0.0.1");
//!
//! let access_control = IpAccessControl::new(config)?;
//!
//! // Check if an IP is allowed
//! if access_control.is_allowed("192.168.1.100") {
//!     println!("Access granted");
//! }
//! ```

pub mod config;
pub mod control;
pub mod middleware;
pub mod types;

#[cfg(test)]
mod tests;

// Re-export main types
pub use config::IpAccessConfig;
pub use control::IpAccessControl;
pub use middleware::{IpAccessMiddleware, IpAccessMiddlewareService};
pub use types::{IpAccessError, IpAccessMode, IpAccessResult, IpRule};
