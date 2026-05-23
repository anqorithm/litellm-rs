use litellm_rs::core::providers::openai::{OpenAIResponseTransformer, models::OpenAIStreamChunk};

#[test]
fn openai_streaming_audio_delta_is_preserved() {
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
                        "data": "base64-audio-delta",
                        "transcript": "hello from audio",
                        "format": "wav"
                    }
                },
                "finish_reason": null
            }
        ]
    }))
    .unwrap();

    let result = OpenAIResponseTransformer::transform_stream_chunk(chunk).unwrap();
    let delta_json = serde_json::to_value(&result.choices[0].delta).unwrap();

    assert_eq!(delta_json["audio"]["data"], "base64-audio-delta");
    assert_eq!(delta_json["audio"]["transcript"], "hello from audio");
    assert_eq!(delta_json["audio"]["format"], "wav");
}
