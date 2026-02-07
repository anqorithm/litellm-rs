//! Perplexity Client
//!
//! Request transformation, response processing, and Perplexity-specific handling
//! including citations and search context support.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::models::get_perplexity_registry;
use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::MessageRole;
use crate::core::types::{ChatRequest, ModelInfo, responses::ChatResponse};

/// Perplexity-specific response with citations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerplexityResponse {
    /// Standard chat response fields
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<PerplexityChoice>,
    pub usage: Option<PerplexityUsage>,

    /// Perplexity-specific: Citations (URLs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citations: Option<Vec<String>>,

    /// Perplexity-specific: Search results with titles
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_results: Option<Vec<SearchResult>>,
}

/// Perplexity choice with message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerplexityChoice {
    pub index: u32,
    pub message: PerplexityMessage,
    pub finish_reason: Option<String>,
}

/// Perplexity message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerplexityMessage {
    pub role: String,
    pub content: String,
}

/// Perplexity usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerplexityUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,

    /// Number of search queries made (Perplexity-specific)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_search_queries: Option<u32>,
}

/// Search result from Perplexity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub url: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
}

/// Citation annotation for response messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub url: String,
    pub title: String,
    pub start_index: usize,
    pub end_index: usize,
}

/// Perplexity API client logic
pub struct PerplexityClient;

impl PerplexityClient {
    /// Transform ChatRequest to Perplexity API format
    pub fn transform_chat_request(request: ChatRequest) -> Value {
        let mut perplexity_request = json!({
            "model": request.model,
            "messages": request.messages,
        });

        // Add standard parameters
        if let Some(temp) = request.temperature {
            perplexity_request["temperature"] = json!(temp);
        }

        if let Some(max_tokens) = request.max_tokens {
            perplexity_request["max_tokens"] = json!(max_tokens);
        }

        if let Some(max_completion_tokens) = request.max_completion_tokens {
            perplexity_request["max_completion_tokens"] = json!(max_completion_tokens);
        }

        if let Some(top_p) = request.top_p {
            perplexity_request["top_p"] = json!(top_p);
        }

        if let Some(freq_penalty) = request.frequency_penalty {
            perplexity_request["frequency_penalty"] = json!(freq_penalty);
        }

        if let Some(pres_penalty) = request.presence_penalty {
            perplexity_request["presence_penalty"] = json!(pres_penalty);
        }

        if request.stream {
            perplexity_request["stream"] = json!(true);
        }

        if let Some(response_format) = &request.response_format {
            perplexity_request["response_format"] = json!(response_format);
        }

        // Handle Perplexity-specific parameters from extra_params
        if let Some(web_search) = request.extra_params.get("web_search_options") {
            perplexity_request["web_search_options"] = web_search.clone();
        }

        if let Some(search_domain_filter) = request.extra_params.get("search_domain_filter") {
            perplexity_request["search_domain_filter"] = search_domain_filter.clone();
        }

        if let Some(search_recency_filter) = request.extra_params.get("search_recency_filter") {
            perplexity_request["search_recency_filter"] = search_recency_filter.clone();
        }

        if let Some(return_citations) = request.extra_params.get("return_citations") {
            perplexity_request["return_citations"] = return_citations.clone();
        }

        if let Some(return_images) = request.extra_params.get("return_images") {
            perplexity_request["return_images"] = return_images.clone();
        }

        if let Some(return_related_questions) = request.extra_params.get("return_related_questions")
        {
            perplexity_request["return_related_questions"] = return_related_questions.clone();
        }

        perplexity_request
    }

    /// Transform Perplexity response to standard ChatResponse
    pub fn transform_chat_response(response: Value) -> Result<ChatResponse, ProviderError> {
        // Check for error response
        if let Some(error) = response.get("error") {
            let error_message = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error from Perplexity API");

            let error_code = error
                .get("code")
                .and_then(|c| c.as_str())
                .unwrap_or("unknown_error");

            return Err(match error_code {
                "authentication_error" | "invalid_api_key" => {
                    ProviderError::authentication("perplexity", error_message)
                }
                "rate_limit_exceeded" => ProviderError::rate_limit("perplexity", None),
                "invalid_model" | "model_not_found" => {
                    ProviderError::model_not_found("perplexity", error_message)
                }
                _ => ProviderError::api_error("perplexity", 400, error_message),
            });
        }

        // Try to parse as Perplexity response first
        if let Ok(perplexity_response) =
            serde_json::from_value::<PerplexityResponse>(response.clone())
        {
            return Self::convert_perplexity_response(perplexity_response);
        }

        // Fall back to direct ChatResponse parsing
        if let Ok(chat_response) = serde_json::from_value::<ChatResponse>(response.clone()) {
            return Ok(chat_response);
        }

        // Build response manually if needed
        let mut response_obj = response
            .as_object()
            .ok_or_else(|| {
                ProviderError::response_parsing(
                    "perplexity",
                    "Response is not an object".to_string(),
                )
            })?
            .clone();

        // Add missing required fields
        if !response_obj.contains_key("id") {
            use std::time::{SystemTime, UNIX_EPOCH};
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();
            response_obj.insert(
                "id".to_string(),
                Value::String(format!("chatcmpl-perplexity-{}", timestamp)),
            );
        }

        if !response_obj.contains_key("object") {
            response_obj.insert(
                "object".to_string(),
                Value::String("chat.completion".to_string()),
            );
        }

        if !response_obj.contains_key("created") {
            use std::time::{SystemTime, UNIX_EPOCH};
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            response_obj.insert(
                "created".to_string(),
                Value::Number(serde_json::Number::from(timestamp)),
            );
        }

        if !response_obj.contains_key("model") {
            response_obj.insert("model".to_string(), Value::String("sonar".to_string()));
        }

        serde_json::from_value(Value::Object(response_obj))
            .map_err(|e| ProviderError::response_parsing("perplexity", e.to_string()))
    }

    /// Convert PerplexityResponse to standard ChatResponse
    fn convert_perplexity_response(
        perplexity: PerplexityResponse,
    ) -> Result<ChatResponse, ProviderError> {
        use crate::core::types::ChatMessage;
        use crate::core::types::MessageContent;
        use crate::core::types::responses::{ChatChoice, FinishReason, Usage};

        let choices: Vec<ChatChoice> = perplexity
            .choices
            .into_iter()
            .map(|choice| {
                let role = match choice.message.role.as_str() {
                    "assistant" => MessageRole::Assistant,
                    "user" => MessageRole::User,
                    "system" => MessageRole::System,
                    "tool" => MessageRole::Tool,
                    _ => MessageRole::Assistant,
                };

                let finish_reason = choice.finish_reason.map(|r| match r.as_str() {
                    "stop" => FinishReason::Stop,
                    "length" => FinishReason::Length,
                    "tool_calls" => FinishReason::ToolCalls,
                    "content_filter" => FinishReason::ContentFilter,
                    _ => FinishReason::Stop,
                });

                ChatChoice {
                    index: choice.index,
                    message: ChatMessage {
                        role,
                        content: Some(MessageContent::Text(choice.message.content)),
                        thinking: None,
                        name: None,
                        tool_calls: None,
                        tool_call_id: None,
                        function_call: None,
                    },
                    finish_reason,
                    logprobs: None,
                }
            })
            .collect();

        let usage = perplexity.usage.map(|u| Usage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
            prompt_tokens_details: None,
            completion_tokens_details: None,
            thinking_usage: None,
        });

        Ok(ChatResponse {
            id: perplexity.id,
            object: perplexity.object,
            created: perplexity.created,
            model: perplexity.model,
            choices,
            usage,
            system_fingerprint: None,
        })
    }

    /// Extract citations from response
    pub fn extract_citations(response: &Value) -> Option<Vec<String>> {
        response
            .get("citations")
            .and_then(|c| c.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
    }

    /// Extract search results from response
    pub fn extract_search_results(response: &Value) -> Option<Vec<SearchResult>> {
        response
            .get("search_results")
            .and_then(|sr| serde_json::from_value::<Vec<SearchResult>>(sr.clone()).ok())
    }

    /// Get all supported models
    pub fn supported_models() -> Vec<ModelInfo> {
        get_perplexity_registry().get_all_models()
    }

    /// Get supported OpenAI parameters for Perplexity
    pub fn supported_openai_params() -> &'static [&'static str] {
        &[
            "frequency_penalty",
            "max_tokens",
            "max_completion_tokens",
            "presence_penalty",
            "response_format",
            "stream",
            "temperature",
            "top_p",
        ]
    }

    /// Parse citation markers from text content
    /// Returns list of (citation_number, start_index, end_index)
    pub fn parse_citation_markers(text: &str) -> Vec<(usize, usize, usize)> {
        let mut citations = Vec::new();
        let mut i = 0;
        let chars: Vec<char> = text.chars().collect();

        while i < chars.len() {
            if chars[i] == '[' {
                let start = i;
                i += 1;
                let mut num_str = String::new();

                while i < chars.len() && chars[i].is_ascii_digit() {
                    num_str.push(chars[i]);
                    i += 1;
                }

                if i < chars.len() && chars[i] == ']' && !num_str.is_empty() {
                    if let Ok(num) = num_str.parse::<usize>() {
                        citations.push((num, start, i + 1));
                    }
                }
            }
            i += 1;
        }

        citations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{ChatMessage, MessageContent, MessageRole};
    use std::collections::HashMap;

    fn create_test_request() -> ChatRequest {
        ChatRequest {
            model: "sonar".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("What is the weather?".to_string())),
                thinking: None,
                name: None,
                tool_calls: None,
                tool_call_id: None,
                function_call: None,
            }],
            temperature: Some(0.7),
            max_tokens: Some(100),
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stream: false,
            tools: None,
            tool_choice: None,
            user: None,
            response_format: None,
            seed: None,
            max_completion_tokens: None,
            stop: None,
            parallel_tool_calls: None,
            n: None,
            logit_bias: None,
            functions: None,
            function_call: None,
            logprobs: None,
            top_logprobs: None,
            thinking: None,
            extra_params: HashMap::new(),
        }
    }

    #[test]
    fn test_transform_request_basic() {
        let request = create_test_request();
        let transformed = PerplexityClient::transform_chat_request(request);

        assert_eq!(transformed["model"], "sonar");
        assert!(transformed["messages"].is_array());
        let temp = transformed["temperature"].as_f64().unwrap();
        assert!((temp - 0.7).abs() < 0.001);
        assert_eq!(transformed["max_tokens"], 100);
    }

    #[test]
    fn test_transform_request_with_streaming() {
        let mut request = create_test_request();
        request.stream = true;
        let transformed = PerplexityClient::transform_chat_request(request);

        assert_eq!(transformed["stream"], true);
    }

    #[test]
    fn test_transform_request_with_search_options() {
        let mut request = create_test_request();
        request.extra_params.insert(
            "search_domain_filter".to_string(),
            json!(["wikipedia.org", "reuters.com"]),
        );
        request
            .extra_params
            .insert("search_recency_filter".to_string(), json!("week"));

        let transformed = PerplexityClient::transform_chat_request(request);

        assert!(transformed["search_domain_filter"].is_array());
        assert_eq!(transformed["search_recency_filter"], "week");
    }

    #[test]
    fn test_transform_response_success() {
        let response = json!({
            "id": "chatcmpl-123",
            "object": "chat.completion",
            "created": 1234567890,
            "model": "sonar",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "The weather is sunny [1]."
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 20,
                "total_tokens": 30
            },
            "citations": ["https://weather.com/forecast"]
        });

        let result = PerplexityClient::transform_chat_response(response);
        assert!(result.is_ok());

        let chat_response = result.unwrap();
        assert_eq!(chat_response.id, "chatcmpl-123");
        assert_eq!(chat_response.model, "sonar");
        assert_eq!(chat_response.choices.len(), 1);
    }

    #[test]
    fn test_transform_response_error() {
        let response = json!({
            "error": {
                "message": "Invalid API key",
                "code": "authentication_error"
            }
        });

        let result = PerplexityClient::transform_chat_response(response);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ProviderError::Authentication { .. }
        ));
    }

    #[test]
    fn test_extract_citations() {
        let response = json!({
            "citations": ["https://example.com", "https://test.com"]
        });

        let citations = PerplexityClient::extract_citations(&response);
        assert!(citations.is_some());
        let citations = citations.unwrap();
        assert_eq!(citations.len(), 2);
        assert_eq!(citations[0], "https://example.com");
    }

    #[test]
    fn test_extract_citations_none() {
        let response = json!({
            "model": "sonar"
        });

        let citations = PerplexityClient::extract_citations(&response);
        assert!(citations.is_none());
    }

    #[test]
    fn test_extract_search_results() {
        let response = json!({
            "search_results": [
                {
                    "url": "https://example.com",
                    "title": "Example Page",
                    "snippet": "This is an example"
                }
            ]
        });

        let results = PerplexityClient::extract_search_results(&response);
        assert!(results.is_some());
        let results = results.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Example Page");
    }

    #[test]
    fn test_supported_models() {
        let models = PerplexityClient::supported_models();
        assert!(models.len() >= 4);

        // Check that sonar model exists
        let sonar = models.iter().find(|m| m.id == "sonar");
        assert!(sonar.is_some());
    }

    #[test]
    fn test_supported_openai_params() {
        let params = PerplexityClient::supported_openai_params();
        assert!(params.contains(&"temperature"));
        assert!(params.contains(&"max_tokens"));
        assert!(params.contains(&"stream"));
        // Perplexity doesn't support tools
        assert!(!params.contains(&"tools"));
    }

    #[test]
    fn test_parse_citation_markers() {
        let text = "According to [1], the weather is sunny. As mentioned in [2] and [3].";
        let citations = PerplexityClient::parse_citation_markers(text);

        assert_eq!(citations.len(), 3);
        assert_eq!(citations[0].0, 1); // citation number
        assert_eq!(citations[1].0, 2);
        assert_eq!(citations[2].0, 3);
    }

    #[test]
    fn test_parse_citation_markers_no_citations() {
        let text = "This text has no citations.";
        let citations = PerplexityClient::parse_citation_markers(text);
        assert!(citations.is_empty());
    }

    #[test]
    fn test_parse_citation_markers_invalid() {
        let text = "This has [invalid] and [] brackets.";
        let citations = PerplexityClient::parse_citation_markers(text);
        assert!(citations.is_empty());
    }

    #[test]
    fn test_perplexity_response_with_search_queries() {
        let response = json!({
            "id": "chatcmpl-123",
            "object": "chat.completion",
            "created": 1234567890,
            "model": "sonar",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Answer"
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 20,
                "total_tokens": 30,
                "num_search_queries": 3
            }
        });

        let result = PerplexityClient::transform_chat_response(response);
        assert!(result.is_ok());
    }

    #[test]
    fn test_convert_perplexity_response() {
        let perplexity_response = PerplexityResponse {
            id: "test-123".to_string(),
            object: "chat.completion".to_string(),
            created: 1234567890,
            model: "sonar".to_string(),
            choices: vec![PerplexityChoice {
                index: 0,
                message: PerplexityMessage {
                    role: "assistant".to_string(),
                    content: "Test response".to_string(),
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: Some(PerplexityUsage {
                prompt_tokens: 10,
                completion_tokens: 20,
                total_tokens: 30,
                num_search_queries: Some(2),
            }),
            citations: Some(vec!["https://example.com".to_string()]),
            search_results: None,
        };

        let result = PerplexityClient::convert_perplexity_response(perplexity_response);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.id, "test-123");
        assert_eq!(response.choices.len(), 1);
    }
}
