//! Codex-specific types, separated by wire and domain responsibility.
pub mod wire {
    use crate::core::models::openai::responses_api::{ResponseInputItem, ResponseTool};
    use serde::de::{DeserializeOwned, Error as DeError};
    use serde::ser::Error as SerError;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use serde_json::{Map, Value};
    /// Codex protocol revision used by the GH-1107 compatibility fixtures.
    pub const CODEX_PROTOCOL_BASELINE: &str = "6e5a2d6b8d148a5554fdceb6f399ca45bd1c78d9";
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CodexFunctionCall {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub id: Option<String>,
        pub call_id: String,
        pub name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub namespace: Option<String>,
        pub arguments: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub status: Option<String>,
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CodexFunctionCallOutput {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub id: Option<String>,
        pub call_id: String,
        pub output: CodexToolOutput,
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CodexCustomToolCall {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub id: Option<String>,
        pub call_id: String,
        pub name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub namespace: Option<String>,
        pub input: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub status: Option<String>,
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CodexCustomToolCallOutput {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub id: Option<String>,
        pub call_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,
        pub output: CodexToolOutput,
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum CodexToolOutput {
        Text(String),
        ContentItems(Vec<CodexToolOutputContent>),
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum CodexToolOutputContent {
        InputText {
            text: String,
        },
        InputImage {
            image_url: String,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            detail: Option<String>,
        },
        InputAudio {
            audio_url: String,
        },
        EncryptedContent {
            encrypted_content: String,
        },
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CodexCustomTool {
        pub name: String,
        pub description: String,
        pub format: Value,
    }
    /// A fail-closed wire item that retains only approved diagnostic metadata.
    #[derive(Debug, Clone)]
    pub struct CodexUnsupportedWire {
        pub wire_type: String,
        metadata: Map<String, Value>,
    }
    impl CodexUnsupportedWire {
        fn new(wire_type: String, payload: Map<String, Value>) -> Self {
            const ALLOWLIST: [&str; 5] = ["id", "call_id", "name", "namespace", "status"];
            let metadata = payload
                .into_iter()
                .filter(|(key, _)| ALLOWLIST.contains(&key.as_str()))
                .collect();
            Self {
                wire_type,
                metadata,
            }
        }
        fn tagged_payload(&self) -> Value {
            let mut payload = self.metadata.clone();
            payload.insert("type".into(), Value::String(self.wire_type.clone()));
            Value::Object(payload)
        }
    }
    impl ResponseInputItem {
        pub fn feature_name(&self) -> &str {
            match self {
                Self::Message(_) => "message",
                Self::FunctionCall(_) => "function_call",
                Self::FunctionCallOutput(_) => "function_call_output",
                Self::CustomToolCall(_) => "custom_tool_call",
                Self::CustomToolCallOutput(_) => "custom_tool_call_output",
                Self::Unsupported(value) | Self::Unknown(value) => &value.wire_type,
            }
        }
    }
    impl Serialize for ResponseInputItem {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let value = match self {
                Self::Message(value) => tag("message", value),
                Self::FunctionCall(value) => tag("function_call", value),
                Self::FunctionCallOutput(value) => tag("function_call_output", value),
                Self::CustomToolCall(value) => tag("custom_tool_call", value),
                Self::CustomToolCallOutput(value) => tag("custom_tool_call_output", value),
                Self::Unsupported(value) | Self::Unknown(value) => Ok(value.tagged_payload()),
            }
            .map_err(S::Error::custom)?;
            value.serialize(serializer)
        }
    }
    impl<'de> Deserialize<'de> for ResponseInputItem {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let (wire_type, payload) = deserialize_tagged(deserializer)?;
            let decoded = match wire_type.as_str() {
                "message" => decode(payload).map(Self::Message),
                "function_call" => decode(payload).map(Self::FunctionCall),
                "function_call_output" => decode(payload).map(Self::FunctionCallOutput),
                "custom_tool_call" => decode(payload).map(Self::CustomToolCall),
                "custom_tool_call_output" => decode(payload).map(Self::CustomToolCallOutput),
                item if is_known_unsupported_item(item) => Ok(Self::Unsupported(
                    CodexUnsupportedWire::new(wire_type, payload),
                )),
                _ => Ok(Self::Unknown(CodexUnsupportedWire::new(wire_type, payload))),
            };
            decoded.map_err(D::Error::custom)
        }
    }
    impl Serialize for ResponseTool {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let value = match self {
                Self::WebSearch(value) => tag("web_search", value),
                Self::WebSearchPreview(value) => tag("web_search_preview", value),
                Self::FileSearch(value) => tag("file_search", value),
                Self::CodeInterpreter(value) => tag("code_interpreter", value),
                Self::ComputerUsePreview(value) => tag("computer_use_preview", value),
                Self::Mcp(value) => tag("mcp", value),
                Self::Function(value) => tag("function", value),
                Self::CodexFunction(value) => tag("function", value),
                Self::Custom(value) => tag("custom", value),
                Self::Unsupported(value) | Self::Unknown(value) => Ok(value.tagged_payload()),
            }
            .map_err(S::Error::custom)?;
            value.serialize(serializer)
        }
    }
    impl<'de> Deserialize<'de> for ResponseTool {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let (wire_type, payload) = deserialize_tagged(deserializer)?;
            let decoded = match wire_type.as_str() {
                "web_search" => decode(payload).map(Self::WebSearch),
                "web_search_preview" => decode(payload).map(Self::WebSearchPreview),
                "file_search" => decode(payload).map(Self::FileSearch),
                "code_interpreter" => decode(payload).map(Self::CodeInterpreter),
                "computer_use_preview" => decode(payload).map(Self::ComputerUsePreview),
                "mcp" => decode(payload).map(Self::Mcp),
                "function" if payload.contains_key("function") => {
                    decode(payload).map(Self::Function)
                }
                "function" => decode(payload).map(Self::CodexFunction),
                "custom" => decode(payload).map(Self::Custom),
                "image_generation" | "namespace" | "tool_search" => Ok(Self::Unsupported(
                    CodexUnsupportedWire::new(wire_type, payload),
                )),
                _ => Ok(Self::Unknown(CodexUnsupportedWire::new(wire_type, payload))),
            };
            decoded.map_err(D::Error::custom)
        }
    }
    fn is_known_unsupported_item(item: &str) -> bool {
        matches!(
            item,
            "additional_tools"
                | "local_shell_call"
                | "mcp_tool_call_output"
                | "tool_search_call"
                | "tool_search_output"
                | "web_search_call"
                | "image_generation_call"
                | "compaction"
                | "compaction_trigger"
                | "context_compaction"
        )
    }
    fn tag<T: Serialize>(wire_type: &str, value: &T) -> Result<Value, serde_json::Error> {
        let Value::Object(mut payload) = serde_json::to_value(value)? else {
            unreachable!("Codex wire payload structs serialize as objects");
        };
        payload.insert("type".into(), Value::String(wire_type.into()));
        Ok(Value::Object(payload))
    }
    fn deserialize_tagged<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<(String, Map<String, Value>), D::Error> {
        let Value::Object(mut payload) = Value::deserialize(deserializer)? else {
            return Err(D::Error::custom("Codex item must be an object"));
        };
        let wire_type = payload
            .remove("type")
            .and_then(|value| value.as_str().map(str::to_owned))
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| D::Error::custom("Codex item type must be a non-empty string"))?;
        Ok((wire_type, payload))
    }

    fn decode<T: DeserializeOwned>(payload: Map<String, Value>) -> Result<T, serde_json::Error> {
        serde_json::from_value(Value::Object(payload))
    }
}
