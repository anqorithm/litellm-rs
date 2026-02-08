//! Core functionality for the Gateway
//!
//! This module contains the core business logic and data structures.

pub mod a2a; // A2A (Agent-to-Agent) Protocol Gateway
pub mod agent; // Agent Coordinator for managing agent lifecycles
pub mod alerting; // Alerting system (Slack, webhooks)
#[cfg(feature = "storage")]
pub mod analytics;
pub mod audio; // Audio API (transcription, translation, speech)
pub mod audit; // Audit logging system
// pub mod base_provider;  // Removed: unused dead code
#[cfg(feature = "storage")]
pub mod batch;
pub mod budget; // Budget management system
// pub mod cache; // DualCache system (InMemory + Redis) - TODO: implement
pub mod cache_manager;
pub mod completion; // Core completion API
pub mod cost; // Unified cost calculation system
pub mod embedding; // Core embedding API (Python LiteLLM compatible)
pub mod fine_tuning; // Fine-tuning API
pub mod function_calling; // Function calling support for AI providers
pub mod guardrails; // Content safety and validation system
pub mod health; // Health monitoring system
pub mod integrations; // External integrations (Langfuse, etc.)
pub mod ip_access; // IP-based access control
pub mod keys; // API Key Management System
pub mod mcp; // MCP (Model Context Protocol) Gateway
pub mod models;
pub mod observability; // Advanced observability and monitoring
pub mod providers;
pub mod rate_limiter; // Rate limiting system
pub mod realtime; // Realtime WebSocket API
pub mod rerank; // Rerank API for RAG systems
pub mod router;
pub mod secret_managers; // Secret management system
pub mod security;
#[cfg(feature = "storage")]
pub mod semantic_cache;
pub mod streaming;
pub mod teams; // Team management module
pub mod traits;
pub mod types;
// User and team management - disabled until database methods are implemented
// These modules require the following database methods to be implemented:
// - virtual_keys: store_virtual_key, get_virtual_key, update_virtual_key, etc.
// - user_management: get_user, create_user, get_team, create_team, etc.
// TODO: Implement database methods and enable these modules
// pub mod user_management;
// pub mod virtual_keys;
pub mod webhooks;
