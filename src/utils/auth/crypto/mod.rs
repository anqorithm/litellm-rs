//! Cryptographic utilities for the Gateway
//!
//! This module provides cryptographic functions for password hashing, API key generation,
//! and authenticated encryption using AES-256-GCM.

pub mod backup;
pub mod encryption;
pub mod hmac;
pub mod keys;
pub mod password;
pub mod webhooks;

#[cfg(test)]
mod tests;
