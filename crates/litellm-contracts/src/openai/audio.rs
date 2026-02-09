//! Audio types for OpenAI-compatible API
//!
//! This module defines audio-related structures for multimodal interactions
//! including audio content, parameters, and delta updates for streaming.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioParams {
    pub voice: String,
    pub format: String,
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct AudioContent {
    pub data: String,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDelta {
    pub data: Option<String>,
    pub transcript: Option<String>,
}
