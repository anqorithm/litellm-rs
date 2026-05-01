//! Shared HTTP utilities for core gateway code.

pub mod outbound;

pub use outbound::{OutboundProfile, build_outbound_client, default_outbound_client};
