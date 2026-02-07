//! LangGraph Models and Types
//!
//! Graph configurations, thread state, and model information for LangGraph Cloud

use crate::core::types::ModelInfo;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

/// Information about a LangGraph graph/assistant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphInfo {
    /// Unique graph identifier
    pub graph_id: String,
    /// Human-readable name
    pub name: String,
    /// Description of the graph's purpose
    pub description: Option<String>,
    /// Version of the graph
    pub version: Option<String>,
    /// Configuration schema (JSON schema)
    pub config_schema: Option<serde_json::Value>,
    /// Input schema for the graph
    pub input_schema: Option<serde_json::Value>,
    /// Output schema for the graph
    pub output_schema: Option<serde_json::Value>,
}

/// Thread state for stateful conversations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThreadState {
    /// Thread ID
    pub thread_id: String,
    /// Current checkpoint ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkpoint_id: Option<String>,
    /// State values (agent memory, context, etc.)
    #[serde(default)]
    pub values: HashMap<String, serde_json::Value>,
    /// Metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
    /// Created timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// Updated timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

/// LangGraph Cloud API request for creating a thread
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateThreadRequest {
    /// Initial metadata for the thread
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// LangGraph Cloud API request for running a graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunGraphRequest {
    /// Assistant ID (graph) to run
    pub assistant_id: String,
    /// Input values for the graph
    pub input: serde_json::Value,
    /// Optional configuration overrides
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
    /// Optional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    /// Whether to stream the response
    #[serde(default)]
    pub stream_mode: Option<Vec<String>>,
    /// Interrupt nodes (for human-in-the-loop)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interrupt_before: Option<Vec<String>>,
    /// Interrupt after nodes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interrupt_after: Option<Vec<String>>,
}

/// LangGraph run response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResponse {
    /// Run ID
    pub run_id: String,
    /// Thread ID
    pub thread_id: String,
    /// Assistant ID used
    pub assistant_id: String,
    /// Status of the run
    pub status: RunStatus,
    /// Output values from the graph
    #[serde(default)]
    pub output: Option<serde_json::Value>,
    /// Error if any
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Created timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// Updated timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

/// Run status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RunStatus {
    /// Run is pending
    Pending,
    /// Run is currently executing
    Running,
    /// Run completed successfully
    Success,
    /// Run failed with an error
    Error,
    /// Run was interrupted
    Interrupted,
    /// Run timed out
    Timeout,
}

impl std::fmt::Display for RunStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunStatus::Pending => write!(f, "pending"),
            RunStatus::Running => write!(f, "running"),
            RunStatus::Success => write!(f, "success"),
            RunStatus::Error => write!(f, "error"),
            RunStatus::Interrupted => write!(f, "interrupted"),
            RunStatus::Timeout => write!(f, "timeout"),
        }
    }
}

/// Get supported LangGraph models (virtual models representing graph types)
pub fn get_langgraph_models() -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            id: "langgraph/agent".to_string(),
            name: "LangGraph Agent".to_string(),
            provider: "langgraph".to_string(),
            max_context_length: 128_000, // Depends on underlying LLM
            max_output_length: Some(16_384),
            supports_streaming: true,
            supports_tools: true,
            supports_multimodal: true, // Depends on graph configuration
            input_cost_per_1k_tokens: None, // Cost depends on underlying providers
            output_cost_per_1k_tokens: None,
            currency: "USD".to_string(),
            capabilities: vec![],
            created_at: None,
            updated_at: None,
            metadata: {
                let mut m = HashMap::new();
                m.insert(
                    "description".to_string(),
                    serde_json::Value::String(
                        "LangGraph agent with tool calling and state management".to_string(),
                    ),
                );
                m
            },
        },
        ModelInfo {
            id: "langgraph/react".to_string(),
            name: "LangGraph ReAct Agent".to_string(),
            provider: "langgraph".to_string(),
            max_context_length: 128_000,
            max_output_length: Some(16_384),
            supports_streaming: true,
            supports_tools: true,
            supports_multimodal: false,
            input_cost_per_1k_tokens: None,
            output_cost_per_1k_tokens: None,
            currency: "USD".to_string(),
            capabilities: vec![],
            created_at: None,
            updated_at: None,
            metadata: {
                let mut m = HashMap::new();
                m.insert(
                    "description".to_string(),
                    serde_json::Value::String(
                        "ReAct (Reasoning + Acting) pattern agent".to_string(),
                    ),
                );
                m
            },
        },
        ModelInfo {
            id: "langgraph/rag".to_string(),
            name: "LangGraph RAG Agent".to_string(),
            provider: "langgraph".to_string(),
            max_context_length: 128_000,
            max_output_length: Some(16_384),
            supports_streaming: true,
            supports_tools: true,
            supports_multimodal: false,
            input_cost_per_1k_tokens: None,
            output_cost_per_1k_tokens: None,
            currency: "USD".to_string(),
            capabilities: vec![],
            created_at: None,
            updated_at: None,
            metadata: {
                let mut m = HashMap::new();
                m.insert(
                    "description".to_string(),
                    serde_json::Value::String(
                        "Retrieval-Augmented Generation agent with vector search".to_string(),
                    ),
                );
                m
            },
        },
        ModelInfo {
            id: "langgraph/supervisor".to_string(),
            name: "LangGraph Supervisor Agent".to_string(),
            provider: "langgraph".to_string(),
            max_context_length: 128_000,
            max_output_length: Some(16_384),
            supports_streaming: true,
            supports_tools: true,
            supports_multimodal: false,
            input_cost_per_1k_tokens: None,
            output_cost_per_1k_tokens: None,
            currency: "USD".to_string(),
            capabilities: vec![],
            created_at: None,
            updated_at: None,
            metadata: {
                let mut m = HashMap::new();
                m.insert(
                    "description".to_string(),
                    serde_json::Value::String(
                        "Multi-agent supervisor for coordinating sub-agents".to_string(),
                    ),
                );
                m
            },
        },
        ModelInfo {
            id: "langgraph/custom".to_string(),
            name: "LangGraph Custom Graph".to_string(),
            provider: "langgraph".to_string(),
            max_context_length: 128_000,
            max_output_length: Some(16_384),
            supports_streaming: true,
            supports_tools: true,
            supports_multimodal: true,
            input_cost_per_1k_tokens: None,
            output_cost_per_1k_tokens: None,
            currency: "USD".to_string(),
            capabilities: vec![],
            created_at: None,
            updated_at: None,
            metadata: {
                let mut m = HashMap::new();
                m.insert(
                    "description".to_string(),
                    serde_json::Value::String(
                        "Custom LangGraph workflow - specify graph_id in config".to_string(),
                    ),
                );
                m
            },
        },
    ]
}

/// Global model registry
static LANGGRAPH_MODELS: OnceLock<Vec<ModelInfo>> = OnceLock::new();

/// Get the global LangGraph model registry
pub fn get_model_registry() -> &'static [ModelInfo] {
    LANGGRAPH_MODELS.get_or_init(get_langgraph_models)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_langgraph_models() {
        let models = get_langgraph_models();
        assert!(!models.is_empty());

        for model in &models {
            assert!(model.id.starts_with("langgraph/"));
            assert_eq!(model.provider, "langgraph");
            assert!(model.supports_streaming);
            assert!(model.supports_tools);
        }
    }

    #[test]
    fn test_thread_state_default() {
        let state = ThreadState::default();
        assert!(state.thread_id.is_empty());
        assert!(state.checkpoint_id.is_none());
        assert!(state.values.is_empty());
    }

    #[test]
    fn test_run_status_display() {
        assert_eq!(format!("{}", RunStatus::Pending), "pending");
        assert_eq!(format!("{}", RunStatus::Running), "running");
        assert_eq!(format!("{}", RunStatus::Success), "success");
        assert_eq!(format!("{}", RunStatus::Error), "error");
    }

    #[test]
    fn test_run_status_equality() {
        assert_eq!(RunStatus::Success, RunStatus::Success);
        assert_ne!(RunStatus::Success, RunStatus::Error);
    }

    #[test]
    fn test_graph_info_serialization() {
        let info = GraphInfo {
            graph_id: "test-graph".to_string(),
            name: "Test Graph".to_string(),
            description: Some("A test graph".to_string()),
            version: Some("1.0".to_string()),
            config_schema: None,
            input_schema: None,
            output_schema: None,
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("test-graph"));
        assert!(json.contains("Test Graph"));
    }

    #[test]
    fn test_create_thread_request() {
        let req = CreateThreadRequest { metadata: None };
        let json = serde_json::to_string(&req).unwrap();
        // Should be minimal JSON since metadata is None
        assert!(json.contains("{}") || json == "{}");
    }

    #[test]
    fn test_run_graph_request() {
        let req = RunGraphRequest {
            assistant_id: "asst-123".to_string(),
            input: serde_json::json!({"messages": [{"role": "user", "content": "Hello"}]}),
            config: None,
            metadata: None,
            stream_mode: Some(vec!["values".to_string()]),
            interrupt_before: None,
            interrupt_after: None,
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("asst-123"));
        assert!(json.contains("messages"));
    }

    #[test]
    fn test_global_model_registry() {
        let models = get_model_registry();
        assert!(!models.is_empty());

        // Should return same reference
        let models2 = get_model_registry();
        assert_eq!(models.len(), models2.len());
    }
}
