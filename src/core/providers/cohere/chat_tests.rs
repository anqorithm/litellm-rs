use super::*;

#[test]
fn test_supported_params() {
    let params = CohereChatHandler::get_supported_params();
    assert!(params.contains(&"temperature"));
    assert!(params.contains(&"max_tokens"));
    assert!(params.contains(&"tools"));
}

#[test]
fn test_map_openai_params() {
    let mut params = HashMap::new();
    params.insert("temperature".to_string(), json!(0.7));
    params.insert("max_tokens".to_string(), json!(100));
    params.insert("top_p".to_string(), json!(0.9));
    params.insert("stop".to_string(), json!(["END"]));

    let mapped = CohereChatHandler::map_openai_params(params);

    assert_eq!(mapped.get("temperature").unwrap(), &json!(0.7));
    assert_eq!(mapped.get("max_tokens").unwrap(), &json!(100));
    assert_eq!(mapped.get("p").unwrap(), &json!(0.9));
    assert_eq!(mapped.get("stop_sequences").unwrap(), &json!(["END"]));
}

#[test]
fn test_extract_usage_v2() {
    let response = json!({
        "usage": {
            "tokens": {
                "input_tokens": 100,
                "output_tokens": 50
            }
        }
    });

    let usage = CohereChatHandler::extract_usage(&response).unwrap();
    assert_eq!(usage.prompt_tokens, 100);
    assert_eq!(usage.completion_tokens, 50);
    assert_eq!(usage.total_tokens, 150);
}

#[test]
fn test_extract_usage_v1() {
    let response = json!({
        "meta": {
            "billed_units": {
                "input_tokens": 80,
                "output_tokens": 40
            }
        }
    });

    let usage = CohereChatHandler::extract_usage(&response).unwrap();
    assert_eq!(usage.prompt_tokens, 80);
    assert_eq!(usage.completion_tokens, 40);
    assert_eq!(usage.total_tokens, 120);
}

#[test]
fn test_extract_content_v2() {
    let response = json!({
        "message": {
            "content": [
                {"type": "text", "text": "Hello, "},
                {"type": "text", "text": "world!"}
            ]
        }
    });

    let content = CohereChatHandler::extract_content(&response).unwrap();
    assert_eq!(content, "Hello, world!");
}

#[test]
fn test_extract_content_v1() {
    let response = json!({
        "text": "Hello from v1!"
    });

    let content = CohereChatHandler::extract_content(&response).unwrap();
    assert_eq!(content, "Hello from v1!");
}

#[test]
fn test_transform_tools_to_v1() {
    let tools = vec![json!({
        "type": "function",
        "function": {
            "name": "get_weather",
            "description": "Get weather info",
            "parameters": {
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "City name"
                    }
                },
                "required": ["location"]
            }
        }
    })];

    let cohere_tools = CohereChatHandler::transform_tools_to_v1(&tools).unwrap();
    let tools_array = cohere_tools.as_array().unwrap();

    assert_eq!(tools_array.len(), 1);
    assert_eq!(tools_array[0]["name"], "get_weather");
    assert!(
        tools_array[0]["parameter_definitions"]["location"]["required"]
            .as_bool()
            .unwrap()
    );
}

#[test]
fn test_cohere_chat_request_serialization() {
    let req = CohereChatRequest {
        model: "command-r-plus".to_string(),
        messages: vec![json!({"role": "user", "content": "hello"})],
        temperature: Some(0.3),
        max_tokens: Some(128),
        p: Some(0.95),
        frequency_penalty: None,
        presence_penalty: None,
        stop_sequences: Some(vec!["END".to_string()]),
        stream: Some(false),
        tools: None,
        seed: Some(42),
        documents: None,
        preamble: None,
    };

    let json = serde_json::to_value(req).unwrap();
    assert_eq!(json["model"], "command-r-plus");
    assert_eq!(json["max_tokens"], 128);
}

#[test]
fn test_cohere_chat_response_roundtrip() {
    let response = CohereChatResponse {
        id: "cohere-response-1".to_string(),
        message: CohereMessage {
            role: "assistant".to_string(),
            content: Some(vec![CohereContent {
                content_type: "text".to_string(),
                text: Some("hello".to_string()),
            }]),
            tool_calls: None,
            citations: Some(vec![CohereCitation {
                start: 0,
                end: 5,
                text: "hello".to_string(),
                sources: vec![CohereSource {
                    source_type: "document".to_string(),
                    id: Some("doc-1".to_string()),
                    document: Some(json!({"title": "doc"})),
                }],
            }]),
        },
        finish_reason: Some("stop".to_string()),
        usage: CohereUsage {
            tokens: CohereTokens {
                input_tokens: 3,
                output_tokens: 2,
            },
        },
    };

    let encoded = serde_json::to_value(&response).unwrap();
    let decoded: CohereChatResponse = serde_json::from_value(encoded).unwrap();
    assert_eq!(decoded.id, "cohere-response-1");
    assert_eq!(decoded.usage.tokens.input_tokens, 3);
    assert_eq!(decoded.usage.tokens.output_tokens, 2);
}
