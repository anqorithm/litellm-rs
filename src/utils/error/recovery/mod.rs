//! Error recovery and resilience utilities
//!
//! This module provides utilities for error recovery, circuit breakers, and resilience patterns.

pub mod circuit_breaker;
pub mod types;

// Include tests module
#[cfg(test)]
mod tests;
