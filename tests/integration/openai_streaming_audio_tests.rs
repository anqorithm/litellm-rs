use litellm_rs::core::providers::openai::{OpenAIResponseTransformer, models::OpenAIStreamChunk};

#[test]
fn openai_streaming_audio_delta_is_preserved() -> Result<(), Box<dyn std::error::Error>> {
    let chunk: OpenAIStreamChunk = serde_json::from_value(serde_json::json!({
        "id": "chatcmpl-audio",
        "object": "chat.completion.chunk",
        "created": 1677652288,
        "model": "gpt-4o-audio-preview",
        "choices": [
            {
                "index": 0,
                "delta": {
                    "audio": {
                        "id": "audio-response-123",
                        "expires_at": 1677655888,
                        "data": "base64-audio-delta",
                        "transcript": "hello from audio",
                        "format": "wav"
                    }
                },
                "finish_reason": null
            }
        ]
    }))?;

    let result = OpenAIResponseTransformer::transform_stream_chunk(chunk)?;
    let delta_json = serde_json::to_value(&result.choices[0].delta)?;

    assert_eq!(delta_json["audio"]["id"], "audio-response-123");
    assert_eq!(delta_json["audio"]["expires_at"], 1677655888);
    assert_eq!(delta_json["audio"]["data"], "base64-audio-delta");
    assert_eq!(delta_json["audio"]["transcript"], "hello from audio");
    assert_eq!(delta_json["audio"]["format"], "wav");
    Ok(())
}

#[test]
fn openai_streaming_reasoning_content_maps_to_thinking() -> Result<(), Box<dyn std::error::Error>> {
    let chunk: OpenAIStreamChunk = serde_json::from_value(serde_json::json!({
        "id": "chatcmpl-reasoning",
        "object": "chat.completion.chunk",
        "created": 1677652288,
        "model": "deepseek-r1",
        "choices": [
            {
                "index": 0,
                "delta": {
                    "reasoning_content": "thinking through the answer"
                },
                "finish_reason": null
            }
        ]
    }))?;

    let result = OpenAIResponseTransformer::transform_stream_chunk(chunk)?;

    assert_eq!(
        result.choices[0].delta.thinking_content(),
        Some("thinking through the answer")
    );
    Ok(())
}
