use super::*;
use serde_json::json;

#[test]
fn test_sse_parsing() {
    let line = r#"data: {"candidates": [{"content": {"parts": [{"text": "Hello"}]}, "finishReason": null}]}"#;
    let event = GeminiSSEParser::parse_event(line);

    assert!(event.is_some());
    match event.unwrap() {
        GeminiSSEEvent::GenerateContentResponse(response) => {
            assert!(response.get("candidates").is_some());
        }
        _ => panic!("Expected GenerateContentResponse"),
    }
}

#[test]
fn test_done_parsing() {
    let line = "data: [DONE]";
    let event = GeminiSSEParser::parse_event(line);

    assert!(event.is_some());
    assert!(matches!(event.unwrap(), GeminiSSEEvent::Done));
}

#[test]
fn test_error_parsing() {
    let line = r#"data: {"error": {"code": 400, "message": "Bad request"}}"#;
    let event = GeminiSSEParser::parse_event(line);

    assert!(event.is_some());
    match event.unwrap() {
        GeminiSSEEvent::Error(error) => {
            assert!(error.get("error").is_some());
        }
        _ => panic!("Expected Error"),
    }
}

#[test]
fn test_chunk_transformation() {
    let response = json!({
        "candidates": [{
            "content": {
                "parts": [{"text": "Hello, world!"}]
            },
            "finishReason": null
        }],
        "usageMetadata": {
            "promptTokenCount": 10,
            "candidatesTokenCount": 5,
            "totalTokenCount": 15
        }
    });

    let event = GeminiSSEEvent::GenerateContentResponse(response);
    let chunk = GeminiSSEParser::transform_to_chat_chunk(&event, "gemini-pro", "test-id")
        .unwrap()
        .unwrap();

    assert_eq!(chunk.model, "gemini-pro");
    assert_eq!(chunk.choices.len(), 1);
    assert_eq!(
        chunk.choices[0].delta.content.as_ref().unwrap(),
        "Hello, world!"
    );
    assert!(chunk.usage.is_some());
}

#[tokio::test]
async fn test_stream_creation() {
    let test_data = vec![
        r#"data: {"candidates": [{"content": {"parts": [{"text": "Hello"}]}}]}"#.to_string(),
        r#"data: {"candidates": [{"content": {"parts": [{"text": " world!"}]}}]}"#.to_string(),
        "data: [DONE]".to_string(),
    ];

    let stream = GeminiStream::from_test_data(test_data, "gemini-pro".to_string());
    let chunks: Vec<_> = stream.collect().await;

    assert_eq!(chunks.len(), 3); // 2 content chunks + 1 completion chunk

    // Check results
    assert!(chunks[0].is_ok());
    assert!(chunks[1].is_ok());
    assert!(chunks[2].is_ok());

    let chunk1 = chunks[0].as_ref().unwrap();
    let chunk2 = chunks[1].as_ref().unwrap();
    let chunk3 = chunks[2].as_ref().unwrap();

    assert_eq!(chunk1.choices[0].delta.content.as_ref().unwrap(), "Hello");
    assert_eq!(chunk2.choices[0].delta.content.as_ref().unwrap(), " world!");
    assert_eq!(
        chunk3.choices[0].finish_reason.as_ref().unwrap(),
        &crate::core::types::responses::FinishReason::Stop
    );
}

#[test]
fn test_sse_empty_line() {
    let event = GeminiSSEParser::parse_event("");
    assert!(event.is_none());
}

#[test]
fn test_sse_comment_line() {
    let event = GeminiSSEParser::parse_event(": this is a comment");
    assert!(event.is_none());
}

#[test]
fn test_sse_event_type_line() {
    let event = GeminiSSEParser::parse_event("event: message");
    assert!(event.is_none());
}

#[test]
fn test_sse_ping_event() {
    let event = GeminiSSEParser::parse_event("data: ");
    assert!(event.is_some());
    assert!(matches!(event.unwrap(), GeminiSSEEvent::Ping));
}

#[test]
fn test_sse_unknown_json() {
    let line = r#"data: {"unknown_field": "value"}"#;
    let event = GeminiSSEParser::parse_event(line);
    assert!(event.is_some());
    assert!(matches!(event.unwrap(), GeminiSSEEvent::Unknown(_)));
}

#[test]
fn test_sse_invalid_json() {
    let line = "data: not valid json";
    let event = GeminiSSEParser::parse_event(line);
    assert!(event.is_some());
    assert!(matches!(event.unwrap(), GeminiSSEEvent::Unknown(_)));
}

#[test]
fn test_chunk_transformation_with_finish_reason_stop() {
    let response = json!({
        "candidates": [{
            "content": {
                "parts": [{"text": "Done."}]
            },
            "finishReason": "STOP"
        }]
    });

    let event = GeminiSSEEvent::GenerateContentResponse(response);
    let chunk = GeminiSSEParser::transform_to_chat_chunk(&event, "gemini-pro", "test-id")
        .unwrap()
        .unwrap();

    assert_eq!(
        chunk.choices[0].finish_reason.as_ref().unwrap(),
        &crate::core::types::responses::FinishReason::Stop
    );
}

#[test]
fn test_chunk_transformation_with_finish_reason_max_tokens() {
    let response = json!({
        "candidates": [{
            "content": {
                "parts": [{"text": "..."}]
            },
            "finishReason": "MAX_TOKENS"
        }]
    });

    let event = GeminiSSEEvent::GenerateContentResponse(response);
    let chunk = GeminiSSEParser::transform_to_chat_chunk(&event, "gemini-pro", "test-id")
        .unwrap()
        .unwrap();

    assert_eq!(
        chunk.choices[0].finish_reason.as_ref().unwrap(),
        &crate::core::types::responses::FinishReason::Length
    );
}

#[test]
fn test_chunk_transformation_with_finish_reason_safety() {
    let response = json!({
        "candidates": [{
            "content": {
                "parts": [{"text": "..."}]
            },
            "finishReason": "SAFETY"
        }]
    });

    let event = GeminiSSEEvent::GenerateContentResponse(response);
    let chunk = GeminiSSEParser::transform_to_chat_chunk(&event, "gemini-pro", "test-id")
        .unwrap()
        .unwrap();

    assert_eq!(
        chunk.choices[0].finish_reason.as_ref().unwrap(),
        &crate::core::types::responses::FinishReason::ContentFilter
    );
}

#[test]
fn test_chunk_transformation_with_finish_reason_recitation() {
    let response = json!({
        "candidates": [{
            "content": {
                "parts": [{"text": "..."}]
            },
            "finishReason": "RECITATION"
        }]
    });

    let event = GeminiSSEEvent::GenerateContentResponse(response);
    let chunk = GeminiSSEParser::transform_to_chat_chunk(&event, "gemini-pro", "test-id")
        .unwrap()
        .unwrap();

    assert_eq!(
        chunk.choices[0].finish_reason.as_ref().unwrap(),
        &crate::core::types::responses::FinishReason::ContentFilter
    );
}

#[test]
fn test_chunk_transformation_ping_event() {
    let event = GeminiSSEEvent::Ping;
    let result = GeminiSSEParser::transform_to_chat_chunk(&event, "gemini-pro", "test-id");
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn test_chunk_transformation_unknown_event() {
    let event = GeminiSSEEvent::Unknown("some data".to_string());
    let result = GeminiSSEParser::transform_to_chat_chunk(&event, "gemini-pro", "test-id");
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn test_chunk_transformation_done_event() {
    let event = GeminiSSEEvent::Done;
    let chunk = GeminiSSEParser::transform_to_chat_chunk(&event, "gemini-pro", "test-id")
        .unwrap()
        .unwrap();

    assert_eq!(chunk.choices.len(), 1);
    assert_eq!(
        chunk.choices[0].finish_reason.as_ref().unwrap(),
        &crate::core::types::responses::FinishReason::Stop
    );
}

#[test]
fn test_chunk_transformation_empty_candidates() {
    let response = json!({
        "candidates": []
    });

    let event = GeminiSSEEvent::GenerateContentResponse(response);
    let result = GeminiSSEParser::transform_to_chat_chunk(&event, "gemini-pro", "test-id");
    assert!(result.is_ok());
    assert!(result.unwrap().is_none()); // Empty candidates should be skipped
}

#[test]
fn test_chunk_transformation_multiple_parts() {
    let response = json!({
        "candidates": [{
            "content": {
                "parts": [
                    {"text": "Hello"},
                    {"text": " "},
                    {"text": "world"}
                ]
            }
        }]
    });

    let event = GeminiSSEEvent::GenerateContentResponse(response);
    let chunk = GeminiSSEParser::transform_to_chat_chunk(&event, "gemini-pro", "test-id")
        .unwrap()
        .unwrap();

    assert_eq!(
        chunk.choices[0].delta.content.as_ref().unwrap(),
        "Hello world"
    );
}

#[test]
fn test_chunk_transformation_multiple_candidates() {
    let response = json!({
        "candidates": [
            {
                "content": {
                    "parts": [{"text": "Response 1"}]
                }
            },
            {
                "content": {
                    "parts": [{"text": "Response 2"}]
                }
            }
        ]
    });

    let event = GeminiSSEEvent::GenerateContentResponse(response);
    let chunk = GeminiSSEParser::transform_to_chat_chunk(&event, "gemini-pro", "test-id")
        .unwrap()
        .unwrap();

    assert_eq!(chunk.choices.len(), 2);
    assert_eq!(chunk.choices[0].index, 0);
    assert_eq!(chunk.choices[1].index, 1);
    assert_eq!(
        chunk.choices[0].delta.content.as_ref().unwrap(),
        "Response 1"
    );
    assert_eq!(
        chunk.choices[1].delta.content.as_ref().unwrap(),
        "Response 2"
    );
}

#[test]
fn test_current_timestamp_secs() {
    let ts = current_timestamp_secs();
    assert!(ts > 0);
}

#[test]
fn test_current_timestamp_nanos() {
    let ts = current_timestamp_nanos();
    assert!(ts > 0);
}
