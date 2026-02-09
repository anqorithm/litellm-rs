    use super::*;

    // ==================== Role Tests ====================

    #[test]
    fn test_role_variants() {
        let system = Role::System;
        let user = Role::User;
        let assistant = Role::Assistant;
        let tool = Role::Tool;

        assert_eq!(system, Role::System);
        assert_eq!(user, Role::User);
        assert_eq!(assistant, Role::Assistant);
        assert_eq!(tool, Role::Tool);
    }

    #[test]
    fn test_role_clone() {
        let role = Role::User;
        let cloned = role.clone();
        assert_eq!(role, cloned);
    }

    #[test]
    fn test_role_serialization() {
        let role = Role::User;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"user\"");

        let system = Role::System;
        let system_json = serde_json::to_string(&system).unwrap();
        assert_eq!(system_json, "\"system\"");

        let assistant = Role::Assistant;
        let assistant_json = serde_json::to_string(&assistant).unwrap();
        assert_eq!(assistant_json, "\"assistant\"");

        let tool = Role::Tool;
        let tool_json = serde_json::to_string(&tool).unwrap();
        assert_eq!(tool_json, "\"tool\"");
    }

    #[test]
    fn test_role_deserialization() {
        let user: Role = serde_json::from_str("\"user\"").unwrap();
        assert_eq!(user, Role::User);

        let system: Role = serde_json::from_str("\"system\"").unwrap();
        assert_eq!(system, Role::System);

        let assistant: Role = serde_json::from_str("\"assistant\"").unwrap();
        assert_eq!(assistant, Role::Assistant);

        let tool: Role = serde_json::from_str("\"tool\"").unwrap();
        assert_eq!(tool, Role::Tool);
    }

    #[test]
    fn test_role_roundtrip() {
        let roles = vec![Role::System, Role::User, Role::Assistant, Role::Tool];
        for role in roles {
            let json = serde_json::to_string(&role).unwrap();
            let deserialized: Role = serde_json::from_str(&json).unwrap();
            assert_eq!(role, deserialized);
        }
    }

    // ==================== Content Tests ====================

    #[test]
    fn test_content_text() {
        let content = Content::Text("Hello, world!".to_string());
        if let Content::Text(text) = content {
            assert_eq!(text, "Hello, world!");
        } else {
            panic!("Expected Text content");
        }
    }

    #[test]
    fn test_content_multimodal() {
        let parts = vec![ContentPart::Text {
            text: "Describe this image".to_string(),
        }];
        let content = Content::Multimodal(parts);
        if let Content::Multimodal(parts) = content {
            assert_eq!(parts.len(), 1);
        } else {
            panic!("Expected Multimodal content");
        }
    }

    #[test]
    fn test_content_text_serialization() {
        let content = Content::Text("Hello".to_string());
        let json = serde_json::to_string(&content).unwrap();
        assert_eq!(json, "\"Hello\"");
    }

    #[test]
    fn test_content_clone() {
        let content = Content::Text("test".to_string());
        let cloned = content.clone();
        if let (Content::Text(a), Content::Text(b)) = (&content, &cloned) {
            assert_eq!(a, b);
        }
    }

    // ==================== ContentPart Tests ====================

    #[test]
    fn test_content_part_text() {
        let part = ContentPart::Text {
            text: "Hello".to_string(),
        };
        if let ContentPart::Text { text } = part {
            assert_eq!(text, "Hello");
        } else {
            panic!("Expected Text part");
        }
    }

    #[test]
    fn test_content_part_image() {
        let part = ContentPart::Image {
            image_url: ImageUrl {
                url: "https://example.com/image.png".to_string(),
                detail: Some("high".to_string()),
            },
        };
        if let ContentPart::Image { image_url } = part {
            assert_eq!(image_url.url, "https://example.com/image.png");
            assert_eq!(image_url.detail, Some("high".to_string()));
        } else {
            panic!("Expected Image part");
        }
    }

    #[test]
    fn test_content_part_audio() {
        let part = ContentPart::Audio {
            audio: AudioData {
                data: "base64data".to_string(),
                format: Some("mp3".to_string()),
            },
        };
        if let ContentPart::Audio { audio } = part {
            assert_eq!(audio.data, "base64data");
            assert_eq!(audio.format, Some("mp3".to_string()));
        } else {
            panic!("Expected Audio part");
        }
    }

    #[test]
    fn test_content_part_text_serialization() {
        let part = ContentPart::Text {
            text: "Hello".to_string(),
        };
        let json = serde_json::to_string(&part).unwrap();
        assert!(json.contains("\"type\":\"text\""));
        assert!(json.contains("\"text\":\"Hello\""));
    }

    #[test]
    fn test_content_part_image_serialization() {
        let part = ContentPart::Image {
            image_url: ImageUrl {
                url: "https://example.com/img.png".to_string(),
                detail: None,
            },
        };
        let json = serde_json::to_string(&part).unwrap();
        assert!(json.contains("\"type\":\"image_url\""));
        assert!(json.contains("\"url\":\"https://example.com/img.png\""));
    }

    // ==================== ImageUrl Tests ====================

    #[test]
    fn test_image_url_creation() {
        let img = ImageUrl {
            url: "https://example.com/image.jpg".to_string(),
            detail: Some("low".to_string()),
        };
        assert_eq!(img.url, "https://example.com/image.jpg");
        assert_eq!(img.detail, Some("low".to_string()));
    }

    #[test]
    fn test_image_url_no_detail() {
        let img = ImageUrl {
            url: "data:image/png;base64,abc123".to_string(),
            detail: None,
        };
        assert!(img.url.starts_with("data:image"));
        assert!(img.detail.is_none());
    }

    #[test]
    fn test_image_url_clone() {
        let img = ImageUrl {
            url: "test.png".to_string(),
            detail: Some("auto".to_string()),
        };
        let cloned = img.clone();
        assert_eq!(img.url, cloned.url);
        assert_eq!(img.detail, cloned.detail);
    }

    // ==================== AudioData Tests ====================

    #[test]
    fn test_audio_data_creation() {
        let audio = AudioData {
            data: "base64encoded".to_string(),
            format: Some("wav".to_string()),
        };
        assert_eq!(audio.data, "base64encoded");
        assert_eq!(audio.format, Some("wav".to_string()));
    }

    #[test]
    fn test_audio_data_no_format() {
        let audio = AudioData {
            data: "audiodata".to_string(),
            format: None,
        };
        assert_eq!(audio.data, "audiodata");
        assert!(audio.format.is_none());
    }

    #[test]
    fn test_audio_data_clone() {
        let audio = AudioData {
            data: "data".to_string(),
            format: Some("mp3".to_string()),
        };
        let cloned = audio.clone();
        assert_eq!(audio.data, cloned.data);
        assert_eq!(audio.format, cloned.format);
    }

    // ==================== Message Tests ====================

    #[test]
    fn test_message_creation() {
        let msg = Message {
            role: Role::User,
            content: Some(Content::Text("Hello".to_string())),
            name: None,
            tool_calls: None,
        };
        assert_eq!(msg.role, Role::User);
        assert!(msg.content.is_some());
        assert!(msg.name.is_none());
        assert!(msg.tool_calls.is_none());
    }

    #[test]
    fn test_message_with_name() {
        let msg = Message {
            role: Role::User,
            content: Some(Content::Text("Hi".to_string())),
            name: Some("John".to_string()),
            tool_calls: None,
        };
        assert_eq!(msg.name, Some("John".to_string()));
    }

    #[test]
    fn test_message_system() {
        let msg = Message {
            role: Role::System,
            content: Some(Content::Text("You are a helpful assistant.".to_string())),
            name: None,
            tool_calls: None,
        };
        assert_eq!(msg.role, Role::System);
    }

    #[test]
    fn test_message_clone() {
        let msg = Message {
            role: Role::Assistant,
            content: Some(Content::Text("Response".to_string())),
            name: None,
            tool_calls: None,
        };
        let cloned = msg.clone();
        assert_eq!(msg.role, cloned.role);
    }

    #[test]
    fn test_message_serialization() {
        let msg = Message {
            role: Role::User,
            content: Some(Content::Text("Hello".to_string())),
            name: None,
            tool_calls: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"role\":\"user\""));
        assert!(json.contains("\"content\":\"Hello\""));
    }

    // ==================== ToolCall Tests ====================

    #[test]
    fn test_tool_call_creation() {
        let call = ToolCall {
            id: "call_123".to_string(),
            tool_type: "function".to_string(),
            function: Function {
                name: "get_weather".to_string(),
                description: None,
                parameters: serde_json::json!({}),
                arguments: Some("{\"city\": \"London\"}".to_string()),
            },
        };
        assert_eq!(call.id, "call_123");
        assert_eq!(call.tool_type, "function");
        assert_eq!(call.function.name, "get_weather");
    }

    #[test]
    fn test_tool_call_clone() {
        let call = ToolCall {
            id: "call_456".to_string(),
            tool_type: "function".to_string(),
            function: Function {
                name: "search".to_string(),
                description: Some("Search the web".to_string()),
                parameters: serde_json::json!({"type": "object"}),
                arguments: None,
            },
        };
        let cloned = call.clone();
        assert_eq!(call.id, cloned.id);
        assert_eq!(call.function.name, cloned.function.name);
    }

    // ==================== Function Tests ====================

    #[test]
    fn test_function_creation() {
        let func = Function {
            name: "calculate".to_string(),
            description: Some("Perform calculations".to_string()),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "expression": {"type": "string"}
                }
            }),
            arguments: None,
        };
        assert_eq!(func.name, "calculate");
        assert!(func.description.is_some());
        assert!(func.arguments.is_none());
    }

    #[test]
    fn test_function_with_arguments() {
        let func = Function {
            name: "greet".to_string(),
            description: None,
            parameters: serde_json::json!({}),
            arguments: Some("{\"name\": \"Alice\"}".to_string()),
        };
        assert_eq!(func.arguments, Some("{\"name\": \"Alice\"}".to_string()));
    }

    #[test]
    fn test_function_clone() {
        let func = Function {
            name: "test".to_string(),
            description: Some("Test function".to_string()),
            parameters: serde_json::json!({}),
            arguments: None,
        };
        let cloned = func.clone();
        assert_eq!(func.name, cloned.name);
        assert_eq!(func.description, cloned.description);
    }

    // ==================== Tool Tests ====================

    #[test]
    fn test_tool_creation() {
        let tool = Tool {
            tool_type: "function".to_string(),
            function: Function {
                name: "get_time".to_string(),
                description: Some("Get current time".to_string()),
                parameters: serde_json::json!({}),
                arguments: None,
            },
        };
        assert_eq!(tool.tool_type, "function");
        assert_eq!(tool.function.name, "get_time");
    }

    #[test]
    fn test_tool_clone() {
        let tool = Tool {
            tool_type: "function".to_string(),
            function: Function {
                name: "search".to_string(),
                description: None,
                parameters: serde_json::json!({}),
                arguments: None,
            },
        };
        let cloned = tool.clone();
        assert_eq!(tool.tool_type, cloned.tool_type);
        assert_eq!(tool.function.name, cloned.function.name);
    }

    // ==================== ChatOptions Tests ====================

    #[test]
    fn test_chat_options_default() {
        let options = ChatOptions::default();
        assert!(options.temperature.is_none());
        assert!(options.max_tokens.is_none());
        assert!(options.top_p.is_none());
        assert!(options.frequency_penalty.is_none());
        assert!(options.presence_penalty.is_none());
        assert!(options.stop.is_none());
        assert!(!options.stream);
        assert!(options.tools.is_none());
        assert!(options.tool_choice.is_none());
    }

    #[test]
    fn test_chat_options_with_values() {
        let options = ChatOptions {
            temperature: Some(0.7),
            max_tokens: Some(1000),
            top_p: Some(0.9),
            frequency_penalty: Some(0.5),
            presence_penalty: Some(0.5),
            stop: Some(vec!["STOP".to_string()]),
            stream: true,
            tools: None,
            tool_choice: None,
        };
        assert_eq!(options.temperature, Some(0.7));
        assert_eq!(options.max_tokens, Some(1000));
        assert_eq!(options.top_p, Some(0.9));
        assert!(options.stream);
    }

    #[test]
    fn test_chat_options_clone() {
        let options = ChatOptions {
            temperature: Some(0.5),
            max_tokens: Some(500),
            ..Default::default()
        };
        let cloned = options.clone();
        assert_eq!(options.temperature, cloned.temperature);
        assert_eq!(options.max_tokens, cloned.max_tokens);
    }

    // ==================== ChatRequest Tests ====================

    #[test]
    fn test_chat_request_creation() {
        let request = ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![Message {
                role: Role::User,
                content: Some(Content::Text("Hello".to_string())),
                name: None,
                tool_calls: None,
            }],
            options: ChatOptions::default(),
        };
        assert_eq!(request.model, "gpt-4");
        assert_eq!(request.messages.len(), 1);
    }

    #[test]
    fn test_chat_request_multiple_messages() {
        let request = ChatRequest {
            model: "claude-3-opus".to_string(),
            messages: vec![
                Message {
                    role: Role::System,
                    content: Some(Content::Text("You are helpful.".to_string())),
                    name: None,
                    tool_calls: None,
                },
                Message {
                    role: Role::User,
                    content: Some(Content::Text("Hi".to_string())),
                    name: None,
                    tool_calls: None,
                },
            ],
            options: ChatOptions::default(),
        };
        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.messages[0].role, Role::System);
        assert_eq!(request.messages[1].role, Role::User);
    }

    #[test]
    fn test_chat_request_clone() {
        let request = ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![],
            options: ChatOptions::default(),
        };
        let cloned = request.clone();
        assert_eq!(request.model, cloned.model);
    }

    // ==================== ChatResponse Tests ====================

    #[test]
    fn test_chat_response_creation() {
        let response = ChatResponse {
            id: "resp_123".to_string(),
            model: "gpt-4".to_string(),
            choices: vec![],
            usage: Usage::default(),
            created: 1234567890,
        };
        assert_eq!(response.id, "resp_123");
        assert_eq!(response.model, "gpt-4");
        assert_eq!(response.created, 1234567890);
    }

    #[test]
    fn test_chat_response_with_choices() {
        let response = ChatResponse {
            id: "resp_456".to_string(),
            model: "gpt-4".to_string(),
            choices: vec![ChatChoice {
                index: 0,
                message: Message {
                    role: Role::Assistant,
                    content: Some(Content::Text("Hello!".to_string())),
                    name: None,
                    tool_calls: None,
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: Usage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            },
            created: 1234567890,
        };
        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.choices[0].index, 0);
        assert_eq!(response.choices[0].finish_reason, Some("stop".to_string()));
    }

    #[test]
    fn test_chat_response_clone() {
        let response = ChatResponse {
            id: "resp_789".to_string(),
            model: "gpt-4".to_string(),
            choices: vec![],
            usage: Usage::default(),
            created: 0,
        };
        let cloned = response.clone();
        assert_eq!(response.id, cloned.id);
    }

    // ==================== ChatChoice Tests ====================

    #[test]
    fn test_chat_choice_creation() {
        let choice = ChatChoice {
            index: 0,
            message: Message {
                role: Role::Assistant,
                content: Some(Content::Text("Response".to_string())),
                name: None,
                tool_calls: None,
            },
            finish_reason: Some("stop".to_string()),
        };
        assert_eq!(choice.index, 0);
        assert_eq!(choice.finish_reason, Some("stop".to_string()));
    }

    #[test]
    fn test_chat_choice_no_finish_reason() {
        let choice = ChatChoice {
            index: 1,
            message: Message {
                role: Role::Assistant,
                content: None,
                name: None,
                tool_calls: None,
            },
            finish_reason: None,
        };
        assert!(choice.finish_reason.is_none());
    }

    // ==================== ChatChunk Tests ====================

    #[test]
    fn test_chat_chunk_creation() {
        let chunk = ChatChunk {
            id: "chunk_123".to_string(),
            model: "gpt-4".to_string(),
            choices: vec![],
        };
        assert_eq!(chunk.id, "chunk_123");
        assert_eq!(chunk.model, "gpt-4");
    }

    #[test]
    fn test_chat_chunk_with_choices() {
        let chunk = ChatChunk {
            id: "chunk_456".to_string(),
            model: "gpt-4".to_string(),
            choices: vec![ChunkChoice {
                index: 0,
                delta: MessageDelta {
                    role: Some(Role::Assistant),
                    content: Some("Hello".to_string()),
                    tool_calls: None,
                },
                finish_reason: None,
            }],
        };
        assert_eq!(chunk.choices.len(), 1);
        assert_eq!(chunk.choices[0].delta.content, Some("Hello".to_string()));
    }

    // ==================== ChunkChoice Tests ====================

    #[test]
    fn test_chunk_choice_creation() {
        let choice = ChunkChoice {
            index: 0,
            delta: MessageDelta {
                role: None,
                content: Some("text".to_string()),
                tool_calls: None,
            },
            finish_reason: None,
        };
        assert_eq!(choice.index, 0);
        assert!(choice.finish_reason.is_none());
    }

    #[test]
    fn test_chunk_choice_with_finish_reason() {
        let choice = ChunkChoice {
            index: 0,
            delta: MessageDelta {
                role: None,
                content: None,
                tool_calls: None,
            },
            finish_reason: Some("stop".to_string()),
        };
        assert_eq!(choice.finish_reason, Some("stop".to_string()));
    }

    // ==================== MessageDelta Tests ====================

    #[test]
    fn test_message_delta_creation() {
        let delta = MessageDelta {
            role: Some(Role::Assistant),
            content: Some("Hello".to_string()),
            tool_calls: None,
        };
        assert_eq!(delta.role, Some(Role::Assistant));
        assert_eq!(delta.content, Some("Hello".to_string()));
    }

    #[test]
    fn test_message_delta_content_only() {
        let delta = MessageDelta {
            role: None,
            content: Some(" world".to_string()),
            tool_calls: None,
        };
        assert!(delta.role.is_none());
        assert_eq!(delta.content, Some(" world".to_string()));
    }

    #[test]
    fn test_message_delta_empty() {
        let delta = MessageDelta {
            role: None,
            content: None,
            tool_calls: None,
        };
        assert!(delta.role.is_none());
        assert!(delta.content.is_none());
        assert!(delta.tool_calls.is_none());
    }

    // ==================== Usage Tests ====================

    #[test]
    fn test_usage_default() {
        let usage = Usage::default();
        assert_eq!(usage.prompt_tokens, 0);
        assert_eq!(usage.completion_tokens, 0);
        assert_eq!(usage.total_tokens, 0);
    }

    #[test]
    fn test_usage_creation() {
        let usage = Usage {
            prompt_tokens: 100,
            completion_tokens: 50,
            total_tokens: 150,
        };
        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.completion_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
    }

    #[test]
    fn test_usage_clone() {
        let usage = Usage {
            prompt_tokens: 10,
            completion_tokens: 20,
            total_tokens: 30,
        };
        let cloned = usage.clone();
        assert_eq!(usage.prompt_tokens, cloned.prompt_tokens);
        assert_eq!(usage.completion_tokens, cloned.completion_tokens);
        assert_eq!(usage.total_tokens, cloned.total_tokens);
    }

    #[test]
    fn test_usage_serialization() {
        let usage = Usage {
            prompt_tokens: 100,
            completion_tokens: 50,
            total_tokens: 150,
        };
        let json = serde_json::to_string(&usage).unwrap();
        assert!(json.contains("\"prompt_tokens\":100"));
        assert!(json.contains("\"completion_tokens\":50"));
        assert!(json.contains("\"total_tokens\":150"));
    }

    #[test]
    fn test_usage_deserialization() {
        let json = r#"{"prompt_tokens":100,"completion_tokens":50,"total_tokens":150}"#;
        let usage: Usage = serde_json::from_str(json).unwrap();
        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.completion_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
    }

    // ==================== Cost Tests ====================

    #[test]
    fn test_cost_creation() {
        let cost = Cost {
            amount: 0.05,
            currency: "USD".to_string(),
            breakdown: CostBreakdown {
                input_cost: 0.03,
                output_cost: 0.02,
                total_cost: 0.05,
            },
        };
        assert_eq!(cost.amount, 0.05);
        assert_eq!(cost.currency, "USD");
    }

    #[test]
    fn test_cost_clone() {
        let cost = Cost {
            amount: 1.0,
            currency: "EUR".to_string(),
            breakdown: CostBreakdown {
                input_cost: 0.6,
                output_cost: 0.4,
                total_cost: 1.0,
            },
        };
        let cloned = cost.clone();
        assert_eq!(cost.amount, cloned.amount);
        assert_eq!(cost.currency, cloned.currency);
    }

    #[test]
    fn test_cost_zero() {
        let cost = Cost {
            amount: 0.0,
            currency: "USD".to_string(),
            breakdown: CostBreakdown {
                input_cost: 0.0,
                output_cost: 0.0,
                total_cost: 0.0,
            },
        };
        assert_eq!(cost.amount, 0.0);
        assert_eq!(cost.breakdown.total_cost, 0.0);
    }

    // ==================== CostBreakdown Tests ====================

    #[test]
    fn test_cost_breakdown_creation() {
        let breakdown = CostBreakdown {
            input_cost: 0.01,
            output_cost: 0.02,
            total_cost: 0.03,
        };
        assert_eq!(breakdown.input_cost, 0.01);
        assert_eq!(breakdown.output_cost, 0.02);
        assert_eq!(breakdown.total_cost, 0.03);
    }

    #[test]
    fn test_cost_breakdown_clone() {
        let breakdown = CostBreakdown {
            input_cost: 0.5,
            output_cost: 0.5,
            total_cost: 1.0,
        };
        let cloned = breakdown.clone();
        assert_eq!(breakdown.input_cost, cloned.input_cost);
        assert_eq!(breakdown.output_cost, cloned.output_cost);
        assert_eq!(breakdown.total_cost, cloned.total_cost);
    }

    #[test]
    fn test_cost_breakdown_debug() {
        let breakdown = CostBreakdown {
            input_cost: 0.1,
            output_cost: 0.2,
            total_cost: 0.3,
        };
        let debug_str = format!("{:?}", breakdown);
        assert!(debug_str.contains("input_cost"));
        assert!(debug_str.contains("output_cost"));
        assert!(debug_str.contains("total_cost"));
    }

    // ==================== ToolChoice Tests ====================

    #[test]
    fn test_tool_choice_variants() {
        let none = ToolChoice::None;
        let auto = ToolChoice::Auto;
        let required = ToolChoice::Required;
        let func = ToolChoice::Function {
            name: "my_function".to_string(),
        };

        // Just verify they can be created
        assert!(matches!(none, ToolChoice::None));
        assert!(matches!(auto, ToolChoice::Auto));
        assert!(matches!(required, ToolChoice::Required));
        assert!(matches!(func, ToolChoice::Function { .. }));
    }

    #[test]
    fn test_tool_choice_function() {
        let choice = ToolChoice::Function {
            name: "get_weather".to_string(),
        };
        if let ToolChoice::Function { name } = choice {
            assert_eq!(name, "get_weather");
        } else {
            panic!("Expected Function variant");
        }
    }

    #[test]
    fn test_tool_choice_clone() {
        let choice = ToolChoice::Function {
            name: "test".to_string(),
        };
        let cloned = choice.clone();
        if let (ToolChoice::Function { name: a }, ToolChoice::Function { name: b }) =
            (&choice, &cloned)
        {
            assert_eq!(a, b);
        }
    }
