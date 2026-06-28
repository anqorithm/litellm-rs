//! Tests for the thinking module
//!
//! Comprehensive tests for all thinking/reasoning provider implementations.

use super::providers::{
    anthropic_thinking, deepseek_thinking, gemini_thinking, openai_thinking, openrouter_thinking,
};
use super::trait_def::{NoThinkingSupport, ThinkingProvider};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::thinking::{
    ThinkingCapabilities, ThinkingConfig, ThinkingContent, ThinkingEffort, ThinkingUsage,
};
use serde_json::Value;

mod anthropic;
mod deepseek;
mod gemini;
mod no_support;
mod openai;
mod openrouter;
mod trait_defaults;
