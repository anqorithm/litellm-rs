//! Metrics collection and aggregation
//!
//! This module provides comprehensive metrics collection for monitoring and observability.

mod background;
mod bounded;
mod collector;
mod getters;
mod helpers;
mod system;
mod types;

#[cfg(test)]
mod tests;

// Re-export the main MetricsCollector struct
pub use collector::MetricsCollector;
