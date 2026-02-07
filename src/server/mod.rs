//! HTTP server implementation
//!
//! This module provides the HTTP server and routing functionality.

// Submodules
pub mod middleware;
pub mod routes;

// New modular server components
pub mod builder;
mod handlers;
pub mod server;
pub mod state;
pub mod types;
mod utils;

pub use server::HttpServer;

#[cfg(test)]
mod tests;
