    use super::*;
    use crate::core::types::{
        content::AudioData, content::DocumentSource, content::ImageSource,
        tools::FunctionDefinition, tools::ToolType,
    };

    // ==================== Request Transformer Tests ====================

    #[test]
    fn test_transform_basic_request() {
        let request = ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello".to_string())),
                ..Default::default()
            }],
            ..Default::default()
        };

        let result = OpenAIRequestTransformer::transform(request);
        assert!(result.is_ok());

        let openai_request = result.unwrap();
        assert_eq!(openai_request.model, "gpt-4");
        assert_eq!(openai_request.messages.len(), 1);
        assert_eq!(openai_request.messages[0].role, "user");
    }

    #[test]
    fn test_transform_request_with_temperature() {
        let request = ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![],
            temperature: Some(0.7),
            top_p: Some(0.9),
            max_tokens: Some(100),
            ..Default::default()
        };

        let result = OpenAIRequestTransformer::transform(request).unwrap();
        assert_eq!(result.temperature, Some(0.7));
        assert_eq!(result.top_p, Some(0.9));
        assert_eq!(result.max_tokens, Some(100));
    }

    #[test]
    fn test_transform_message_roles() {
        let messages = vec![
            ChatMessage {
                role: MessageRole::System,
                content: Some(MessageContent::Text("You are helpful".to_string())),
                ..Default::default()
            },
            ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello".to_string())),
                ..Default::default()
            },
            ChatMessage {
                role: MessageRole::Assistant,
                content: Some(MessageContent::Text("Hi there".to_string())),
                ..Default::default()
            },
            ChatMessage {
                role: MessageRole::Tool,
                content: Some(MessageContent::Text("result".to_string())),
                tool_call_id: Some("tool-123".to_string()),
                ..Default::default()
            },
        ];

        let request = ChatRequest {
            model: "gpt-4".to_string(),
            messages,
            ..Default::default()
        };

        let result = OpenAIRequestTransformer::transform(request).unwrap();
        assert_eq!(result.messages[0].role, "system");
        assert_eq!(result.messages[1].role, "user");
        assert_eq!(result.messages[2].role, "assistant");
        assert_eq!(result.messages[3].role, "tool");
    }

    #[test]
    fn test_transform_content_parts_text() {
        let request = ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Parts(vec![ContentPart::Text {
                    text: "Hello world".to_string(),
                }])),
                ..Default::default()
            }],
            ..Default::default()
        };

        let result = OpenAIRequestTransformer::transform(request).unwrap();
        assert!(result.messages[0].content.is_some());
    }

    #[test]
    fn test_transform_content_parts_image_url() {
        let request = ChatRequest {
            model: "gpt-4-vision".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Parts(vec![
                    ContentPart::Text {
                        text: "What's in this image?".to_string(),
                    },
                    ContentPart::ImageUrl {
                        image_url: ImageUrl {
                            url: "https://example.com/image.png".to_string(),
                            detail: Some("high".to_string()),
                        },
                    },
                ])),
                ..Default::default()
            }],
            ..Default::default()
        };

        let result = OpenAIRequestTransformer::transform(request).unwrap();
        assert!(result.messages[0].content.is_some());
    }

    #[test]
    fn test_transform_content_parts_audio() {
        let request = ChatRequest {
            model: "gpt-4o-audio".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Parts(vec![ContentPart::Audio {
                    audio: AudioData {
                        data: "base64data".to_string(),
                        format: Some("mp3".to_string()),
                    },
                }])),
                ..Default::default()
            }],
            ..Default::default()
        };

        let result = OpenAIRequestTransformer::transform(request).unwrap();
        assert!(result.messages[0].content.is_some());
    }

    #[test]
    fn test_transform_content_parts_image_source() {
        let request = ChatRequest {
            model: "gpt-4-vision".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Parts(vec![ContentPart::Image {
                    source: ImageSource {
                        media_type: "image/png".to_string(),
                        data: "base64imagedata".to_string(),
                    },
                    detail: Some("high".to_string()),
                    image_url: None,
                }])),
                ..Default::default()
            }],
            ..Default::default()
        };

        let result = OpenAIRequestTransformer::transform(request).unwrap();
        assert!(result.messages[0].content.is_some());
    }

    #[test]
    fn test_transform_document_content_error() {
        let request = ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Parts(vec![ContentPart::Document {
                    source: DocumentSource {
                        media_type: "application/pdf".to_string(),
                        data: "base64pdfdata".to_string(),
                    },
                    cache_control: None,
                }])),
                ..Default::default()
            }],
            ..Default::default()
        };

        let result = OpenAIRequestTransformer::transform(request);
        assert!(result.is_err());
    }

    #[test]
    fn test_transform_tool_call() {
        let request = ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::Assistant,
                content: None,
                tool_calls: Some(vec![ToolCall {
                    id: "call_123".to_string(),
                    tool_type: "function".to_string(),
                    function: FunctionCall {
                        name: "get_weather".to_string(),
                        arguments: r#"{"location":"NYC"}"#.to_string(),
                    },
                }]),
                ..Default::default()
            }],
            ..Default::default()
        };

        let result = OpenAIRequestTransformer::transform(request).unwrap();
        let tool_calls = result.messages[0].tool_calls.as_ref().unwrap();
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].id, "call_123");
        assert_eq!(tool_calls[0].function.name, "get_weather");
    }

    #[test]
    fn test_transform_tools() {
        let request = ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![],
            tools: Some(vec![Tool {
                tool_type: ToolType::Function,
                function: FunctionDefinition {
                    name: "get_weather".to_string(),
                    description: Some("Get weather info".to_string()),
                    parameters: Some(serde_json::json!({
                        "type": "object",
                        "properties": {
                            "location": {"type": "string"}
                        }
                    })),
                },
            }]),
            ..Default::default()
        };

        let result = OpenAIRequestTransformer::transform(request).unwrap();
        let tools = result.tools.unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].function.as_ref().unwrap().name, "get_weather");
    }

    #[test]
    fn test_transform_tool_choice_none() {
        let request = ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![],
            tool_choice: Some(ToolChoice::String("none".to_string())),
            ..Default::default()
        };

        let result = OpenAIRequestTransformer::transform(request).unwrap();
        assert!(result.tool_choice.is_some());
    }

    #[test]
    fn test_transform_tool_choice_auto() {
        let request = ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![],
            tool_choice: Some(ToolChoice::String("auto".to_string())),
            ..Default::default()
        };

        let result = OpenAIRequestTransformer::transform(request).unwrap();
        assert!(result.tool_choice.is_some());
    }

    #[test]
    fn test_transform_tool_choice_required() {
        let request = ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![],
            tool_choice: Some(ToolChoice::String("required".to_string())),
            ..Default::default()
        };

        let result = OpenAIRequestTransformer::transform(request).unwrap();
        assert!(result.tool_choice.is_some());
    }

    #[test]
    fn test_transform_tool_choice_specific() {
        let request = ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![],
            tool_choice: Some(ToolChoice::Specific {
                choice_type: "function".to_string(),
                function: Some(crate::core::types::tools::FunctionChoice {
                    name: "get_weather".to_string(),
                }),
            }),
            ..Default::default()
        };

        let result = OpenAIRequestTransformer::transform(request).unwrap();
        assert!(result.tool_choice.is_some());
    }

    #[test]
    fn test_transform_response_format() {
        let request = ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![],
            response_format: Some(ResponseFormat {
                format_type: "json_object".to_string(),
                json_schema: None,
                response_type: None,
            }),
            ..Default::default()
        };

        let result = OpenAIRequestTransformer::transform(request).unwrap();
        let format = result.response_format.unwrap();
        assert_eq!(format.format_type, "json_object");
    }

    #[test]
    fn test_transform_response_format_with_schema() {
        let request = ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![],
            response_format: Some(ResponseFormat {
                format_type: "json_schema".to_string(),
                json_schema: Some(serde_json::json!({
                    "name": "response",
                    "schema": {"type": "object"}
                })),
                response_type: None,
            }),
            ..Default::default()
        };

        let result = OpenAIRequestTransformer::transform(request).unwrap();
        let format = result.response_format.unwrap();
        assert_eq!(format.format_type, "json_schema");
        assert!(format.json_schema.is_some());
    }

    // ==================== Response Transformer Tests ====================

    #[test]
    fn test_transform_basic_response() {
        let response = OpenAIChatResponse {
            id: "chatcmpl-123".to_string(),
            object: "chat.completion".to_string(),
            created: 1677652288,
            model: "gpt-4".to_string(),
            choices: vec![OpenAIChoice {
                index: 0,
                message: OpenAIMessage {
                    role: "assistant".to_string(),
                    content: Some(serde_json::json!("Hello!")),
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                    function_call: None,
                    reasoning: None,
                    reasoning_details: None,
                    reasoning_content: None,
                },
                finish_reason: Some("stop".to_string()),
                logprobs: None,
            }],
            usage: Some(OpenAIUsage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
                prompt_tokens_details: None,
                completion_tokens_details: None,
            }),
            system_fingerprint: Some("fp_123".to_string()),
        };

        let result = OpenAIResponseTransformer::transform(response).unwrap();
        assert_eq!(result.id, "chatcmpl-123");
        assert_eq!(result.model, "gpt-4");
        assert_eq!(result.choices.len(), 1);
        assert!(matches!(
            result.choices[0].finish_reason,
            Some(FinishReason::Stop)
        ));
    }

    #[test]
    fn test_transform_response_with_usage_details() {
        let response = OpenAIChatResponse {
            id: "chatcmpl-123".to_string(),
            object: "chat.completion".to_string(),
            created: 1677652288,
            model: "gpt-4".to_string(),
            choices: vec![],
            usage: Some(OpenAIUsage {
                prompt_tokens: 100,
                completion_tokens: 50,
                total_tokens: 150,
                prompt_tokens_details: Some(OpenAITokenDetails {
                    cached_tokens: Some(20),
                    audio_tokens: Some(5),
                    reasoning_tokens: None,
                }),
                completion_tokens_details: Some(OpenAITokenDetails {
                    cached_tokens: None,
                    audio_tokens: Some(10),
                    reasoning_tokens: Some(15),
                }),
            }),
            system_fingerprint: None,
        };

        let result = OpenAIResponseTransformer::transform(response).unwrap();
        let usage = result.usage.unwrap();
        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.completion_tokens, 50);
        assert_eq!(
            usage.prompt_tokens_details.as_ref().unwrap().cached_tokens,
            Some(20)
        );
        assert_eq!(
            usage
                .completion_tokens_details
                .as_ref()
                .unwrap()
                .reasoning_tokens,
            Some(15)
        );
    }

    #[test]
    fn test_transform_response_role_mapping() {
        let roles = vec!["system", "user", "assistant", "tool", "function", "unknown"];

        for role in roles {
            let response = OpenAIChatResponse {
                id: "test".to_string(),
                object: "chat.completion".to_string(),
                created: 0,
                model: "gpt-4".to_string(),
                choices: vec![OpenAIChoice {
                    index: 0,
                    message: OpenAIMessage {
                        role: role.to_string(),
                        content: Some(serde_json::json!("test")),
                        name: None,
                        tool_calls: None,
                        tool_call_id: None,
                        function_call: None,
                        reasoning: None,
                        reasoning_details: None,
                        reasoning_content: None,
                    },
                    finish_reason: None,
                    logprobs: None,
                }],
                usage: None,
                system_fingerprint: None,
            };

            let result = OpenAIResponseTransformer::transform(response);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_transform_finish_reasons() {
        let reasons = vec![
            ("stop", FinishReason::Stop),
            ("length", FinishReason::Length),
            ("function_call", FinishReason::FunctionCall),
            ("tool_calls", FinishReason::ToolCalls),
            ("content_filter", FinishReason::ContentFilter),
            ("unknown", FinishReason::Stop), // Default fallback
        ];

        for (reason_str, expected) in reasons {
            let response = OpenAIChatResponse {
                id: "test".to_string(),
                object: "chat.completion".to_string(),
                created: 0,
                model: "gpt-4".to_string(),
                choices: vec![OpenAIChoice {
                    index: 0,
                    message: OpenAIMessage {
                        role: "assistant".to_string(),
                        content: None,
                        name: None,
                        tool_calls: None,
                        tool_call_id: None,
                        function_call: None,
                        reasoning: None,
                        reasoning_details: None,
                        reasoning_content: None,
                    },
                    finish_reason: Some(reason_str.to_string()),
                    logprobs: None,
                }],
                usage: None,
                system_fingerprint: None,
            };

            let result = OpenAIResponseTransformer::transform(response).unwrap();
            assert_eq!(result.choices[0].finish_reason, Some(expected));
        }
    }

    #[test]
    fn test_transform_response_with_tool_calls() {
        let response = OpenAIChatResponse {
            id: "test".to_string(),
            object: "chat.completion".to_string(),
            created: 0,
            model: "gpt-4".to_string(),
            choices: vec![OpenAIChoice {
                index: 0,
                message: OpenAIMessage {
                    role: "assistant".to_string(),
                    content: None,
                    name: None,
                    tool_calls: Some(vec![OpenAIToolCall {
                        id: "call_abc".to_string(),
                        tool_type: "function".to_string(),
                        function: OpenAIFunctionCall {
                            name: "get_weather".to_string(),
                            arguments: r#"{"location":"NYC"}"#.to_string(),
                        },
                    }]),
                    tool_call_id: None,
                    function_call: None,
                    reasoning: None,
                    reasoning_details: None,
                    reasoning_content: None,
                },
                finish_reason: Some("tool_calls".to_string()),
                logprobs: None,
            }],
            usage: None,
            system_fingerprint: None,
        };

        let result = OpenAIResponseTransformer::transform(response).unwrap();
        let tool_calls = result.choices[0].message.tool_calls.as_ref().unwrap();
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].id, "call_abc");
        assert_eq!(tool_calls[0].function.name, "get_weather");
    }

    #[test]
    fn test_transform_response_with_reasoning() {
        let response = OpenAIChatResponse {
            id: "test".to_string(),
            object: "chat.completion".to_string(),
            created: 0,
            model: "o1-preview".to_string(),
            choices: vec![OpenAIChoice {
                index: 0,
                message: OpenAIMessage {
                    role: "assistant".to_string(),
                    content: Some(serde_json::json!("The answer is 42")),
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                    function_call: None,
                    reasoning: Some("Let me think about this...".to_string()),
                    reasoning_details: None,
                    reasoning_content: None,
                },
                finish_reason: Some("stop".to_string()),
                logprobs: None,
            }],
            usage: None,
            system_fingerprint: None,
        };

        let result = OpenAIResponseTransformer::transform(response).unwrap();
        assert!(result.choices[0].message.thinking.is_some());
    }

    #[test]
    fn test_transform_response_with_deepseek_reasoning() {
        let response = OpenAIChatResponse {
            id: "test".to_string(),
            object: "chat.completion".to_string(),
            created: 0,
            model: "deepseek-chat".to_string(),
            choices: vec![OpenAIChoice {
                index: 0,
                message: OpenAIMessage {
                    role: "assistant".to_string(),
                    content: Some(serde_json::json!("Result")),
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                    function_call: None,
                    reasoning: None,
                    reasoning_details: None,
                    reasoning_content: Some("DeepSeek thinking process...".to_string()),
                },
                finish_reason: Some("stop".to_string()),
                logprobs: None,
            }],
            usage: None,
            system_fingerprint: None,
        };

        let result = OpenAIResponseTransformer::transform(response).unwrap();
        assert!(result.choices[0].message.thinking.is_some());
    }

    #[test]
    fn test_transform_response_null_content() {
        let response = OpenAIChatResponse {
            id: "test".to_string(),
            object: "chat.completion".to_string(),
            created: 0,
            model: "gpt-4".to_string(),
            choices: vec![OpenAIChoice {
                index: 0,
                message: OpenAIMessage {
                    role: "assistant".to_string(),
                    content: Some(serde_json::Value::Null),
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                    function_call: None,
                    reasoning: None,
                    reasoning_details: None,
                    reasoning_content: None,
                },
                finish_reason: None,
                logprobs: None,
            }],
            usage: None,
            system_fingerprint: None,
        };

        let result = OpenAIResponseTransformer::transform(response).unwrap();
        assert!(result.choices[0].message.content.is_none());
    }

    #[test]
    fn test_transform_response_empty_content() {
        let response = OpenAIChatResponse {
            id: "test".to_string(),
            object: "chat.completion".to_string(),
            created: 0,
            model: "gpt-4".to_string(),
            choices: vec![OpenAIChoice {
                index: 0,
                message: OpenAIMessage {
                    role: "assistant".to_string(),
                    content: Some(serde_json::json!("")),
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                    function_call: None,
                    reasoning: None,
                    reasoning_details: None,
                    reasoning_content: None,
                },
                finish_reason: None,
                logprobs: None,
            }],
            usage: None,
            system_fingerprint: None,
        };

        let result = OpenAIResponseTransformer::transform(response).unwrap();
        assert!(result.choices[0].message.content.is_none());
    }

    // ==================== Stream Transformer Tests ====================

    #[test]
    fn test_transform_stream_chunk() {
        let chunk = OpenAIStreamChunk {
            id: "chatcmpl-123".to_string(),
            object: "chat.completion.chunk".to_string(),
            created: 1677652288,
            model: "gpt-4".to_string(),
            choices: vec![OpenAIStreamChoice {
                index: 0,
                delta: OpenAIDelta {
                    role: Some("assistant".to_string()),
                    content: Some("Hello".to_string()),
                    tool_calls: None,
                    function_call: None,
                },
                finish_reason: None,
                logprobs: None,
            }],
            usage: None,
            system_fingerprint: None,
        };

        let result = OpenAIResponseTransformer::transform_stream_chunk(chunk).unwrap();
        assert_eq!(result.id, "chatcmpl-123");
        assert_eq!(result.choices.len(), 1);
        assert_eq!(result.choices[0].delta.content, Some("Hello".to_string()));
    }

    #[test]
    fn test_transform_stream_chunk_with_finish() {
        let chunk = OpenAIStreamChunk {
            id: "chatcmpl-123".to_string(),
            object: "chat.completion.chunk".to_string(),
            created: 1677652288,
            model: "gpt-4".to_string(),
            choices: vec![OpenAIStreamChoice {
                index: 0,
                delta: OpenAIDelta {
                    role: None,
                    content: None,
                    tool_calls: None,
                    function_call: None,
                },
                finish_reason: Some("stop".to_string()),
                logprobs: None,
            }],
            usage: Some(OpenAIUsage {
                prompt_tokens: 10,
                completion_tokens: 20,
                total_tokens: 30,
                prompt_tokens_details: None,
                completion_tokens_details: None,
            }),
            system_fingerprint: None,
        };

        let result = OpenAIResponseTransformer::transform_stream_chunk(chunk).unwrap();
        assert!(matches!(
            result.choices[0].finish_reason,
            Some(FinishReason::Stop)
        ));
        assert!(result.usage.is_some());
    }

    #[test]
    fn test_transform_delta_roles() {
        let roles = vec!["system", "user", "assistant", "tool", "function", "unknown"];

        for role in roles {
            let delta = OpenAIDelta {
                role: Some(role.to_string()),
                content: None,
                tool_calls: None,
                function_call: None,
            };

            let result = OpenAIResponseTransformer::transform_delta(delta);
            assert!(result.is_ok());
        }
    }

    // ==================== Trait Implementation Tests ====================

    #[test]
    fn test_transform_trait_request() {
        let request = ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![],
            ..Default::default()
        };

        let result =
            <OpenAITransformer as Transform<ChatRequest, OpenAIChatRequest>>::transform(request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transform_trait_response() {
        let response = OpenAIChatResponse {
            id: "test".to_string(),
            object: "chat.completion".to_string(),
            created: 0,
            model: "gpt-4".to_string(),
            choices: vec![],
            usage: None,
            system_fingerprint: None,
        };

        let result =
            <OpenAITransformer as Transform<OpenAIChatResponse, ChatResponse>>::transform(response);
        assert!(result.is_ok());
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_transform_request_with_all_optional_fields() {
        let request = ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("test".to_string())),
                name: Some("test_user".to_string()),
                ..Default::default()
            }],
            temperature: Some(0.5),
            top_p: Some(0.9),
            n: Some(2),
            stop: Some(vec!["END".to_string()]),
            max_tokens: Some(500),
            max_completion_tokens: Some(400),
            presence_penalty: Some(0.5),
            frequency_penalty: Some(0.3),
            logprobs: Some(true),
            top_logprobs: Some(5),
            user: Some("user123".to_string()),
            seed: Some(42),
            ..Default::default()
        };

        let result = OpenAIRequestTransformer::transform(request).unwrap();
        assert_eq!(result.temperature, Some(0.5));
        assert_eq!(result.n, Some(2));
        assert_eq!(result.seed, Some(42));
        assert_eq!(result.user, Some("user123".to_string()));
    }

    #[test]
    fn test_transform_response_with_logprobs() {
        let response = OpenAIChatResponse {
            id: "test".to_string(),
            object: "chat.completion".to_string(),
            created: 0,
            model: "gpt-4".to_string(),
            choices: vec![OpenAIChoice {
                index: 0,
                message: OpenAIMessage {
                    role: "assistant".to_string(),
                    content: Some(serde_json::json!("test")),
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                    function_call: None,
                    reasoning: None,
                    reasoning_details: None,
                    reasoning_content: None,
                },
                finish_reason: Some("stop".to_string()),
                logprobs: Some(serde_json::json!({
                    "content": [{
                        "token": "test",
                        "logprob": -0.5,
                        "bytes": [116, 101, 115, 116],
                        "top_logprobs": [{
                            "token": "test",
                            "logprob": -0.5,
                            "bytes": [116, 101, 115, 116]
                        }]
                    }]
                })),
            }],
            usage: None,
            system_fingerprint: None,
        };

        let result = OpenAIResponseTransformer::transform(response).unwrap();
        assert!(result.choices[0].logprobs.is_some());
    }

    #[test]
    fn test_transform_response_content_array() {
        let response = OpenAIChatResponse {
            id: "test".to_string(),
            object: "chat.completion".to_string(),
            created: 0,
            model: "gpt-4".to_string(),
            choices: vec![OpenAIChoice {
                index: 0,
                message: OpenAIMessage {
                    role: "assistant".to_string(),
                    content: Some(serde_json::json!([
                        {"type": "text", "text": "Hello"}
                    ])),
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                    function_call: None,
                    reasoning: None,
                    reasoning_details: None,
                    reasoning_content: None,
                },
                finish_reason: None,
                logprobs: None,
            }],
            usage: None,
            system_fingerprint: None,
        };

        let result = OpenAIResponseTransformer::transform(response).unwrap();
        assert!(result.choices[0].message.content.is_some());
    }

    #[test]
    fn test_transform_message_with_function_call() {
        let request = ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::Assistant,
                content: None,
                function_call: Some(FunctionCall {
                    name: "get_weather".to_string(),
                    arguments: r#"{"location":"NYC"}"#.to_string(),
                }),
                ..Default::default()
            }],
            ..Default::default()
        };

        let result = OpenAIRequestTransformer::transform(request).unwrap();
        assert!(result.messages[0].function_call.is_some());
    }

    #[test]
    fn test_transform_response_with_function_call() {
        let response = OpenAIChatResponse {
            id: "test".to_string(),
            object: "chat.completion".to_string(),
            created: 0,
            model: "gpt-4".to_string(),
            choices: vec![OpenAIChoice {
                index: 0,
                message: OpenAIMessage {
                    role: "assistant".to_string(),
                    content: None,
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                    function_call: Some(OpenAIFunctionCall {
                        name: "get_weather".to_string(),
                        arguments: r#"{"location":"NYC"}"#.to_string(),
                    }),
                    reasoning: None,
                    reasoning_details: None,
                    reasoning_content: None,
                },
                finish_reason: Some("function_call".to_string()),
                logprobs: None,
            }],
            usage: None,
            system_fingerprint: None,
        };

        let result = OpenAIResponseTransformer::transform(response).unwrap();
        let func_call = result.choices[0].message.function_call.as_ref().unwrap();
        assert_eq!(func_call.name, "get_weather");
    }
