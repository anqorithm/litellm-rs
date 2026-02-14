//! Monitoring and observability system
//!
//! This module provides comprehensive monitoring, metrics, and observability functionality.

// Public submodules
pub mod alerts;
pub mod health;
pub mod metrics;
pub mod system;
pub mod types;

// Internal submodules
mod background;
#[cfg(test)]
mod tests;
