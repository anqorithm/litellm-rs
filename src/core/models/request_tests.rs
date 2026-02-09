use super::*;

#[test]
fn test_gateway_request_creation() {
    let context = RequestContext::new();
    let chat_request = ChatCompletionRequest::default();
    let data = RequestData::ChatCompletion(Box::new(chat_request));

    let gateway_request = GatewayRequest::new(RequestType::ChatCompletion, data, context);

    assert!(matches!(
        gateway_request.request_type,
        RequestType::ChatCompletion
    ));
    assert!(matches!(
        gateway_request.data,
        RequestData::ChatCompletion(_)
    ));
}

#[test]
fn test_model_extraction() {
    let context = RequestContext::new();
    let chat_request = ChatCompletionRequest {
        model: "gpt-4".to_string(),
        ..Default::default()
    };
    let data = RequestData::ChatCompletion(Box::new(chat_request));

    let gateway_request = GatewayRequest::new(RequestType::ChatCompletion, data, context);

    assert_eq!(gateway_request.model(), Some("gpt-4"));
}

#[test]
fn test_streaming_detection() {
    let context = RequestContext::new();
    let chat_request = ChatCompletionRequest {
        stream: Some(true),
        ..Default::default()
    };
    let data = RequestData::ChatCompletion(Box::new(chat_request));

    let gateway_request = GatewayRequest::new(RequestType::ChatCompletion, data, context);

    assert!(gateway_request.is_streaming());
}

// ==================== RequestType Tests ====================

#[test]
fn test_request_type_serialization() {
    assert_eq!(
        serde_json::to_string(&RequestType::ChatCompletion).unwrap(),
        "\"chat_completion\""
    );
    assert_eq!(
        serde_json::to_string(&RequestType::Embedding).unwrap(),
        "\"embedding\""
    );
    assert_eq!(
        serde_json::to_string(&RequestType::ImageGeneration).unwrap(),
        "\"image_generation\""
    );
}

#[test]
fn test_request_type_deserialization() {
    let chat: RequestType = serde_json::from_str("\"chat_completion\"").unwrap();
    assert!(matches!(chat, RequestType::ChatCompletion));

    let embed: RequestType = serde_json::from_str("\"embedding\"").unwrap();
    assert!(matches!(embed, RequestType::Embedding));
}

#[test]
fn test_request_type_all_variants() {
    let types = vec![
        RequestType::ChatCompletion,
        RequestType::Completion,
        RequestType::Embedding,
        RequestType::ImageGeneration,
        RequestType::ImageEdit,
        RequestType::ImageVariation,
        RequestType::AudioTranscription,
        RequestType::AudioTranslation,
        RequestType::AudioSpeech,
        RequestType::Moderation,
        RequestType::FineTuning,
        RequestType::Files,
        RequestType::Assistants,
        RequestType::Threads,
        RequestType::Batches,
        RequestType::VectorStores,
        RequestType::Rerank,
        RequestType::Realtime,
    ];
    assert_eq!(types.len(), 18);
}

// ==================== RoutingPreferences Tests ====================

#[test]
fn test_routing_preferences_default() {
    let prefs = RoutingPreferences::default();
    assert!(prefs.preferred_providers.is_empty());
    assert!(prefs.excluded_providers.is_empty());
    assert!(prefs.strategy_override.is_none());
    assert!(prefs.tags.is_empty());
    assert!(prefs.region.is_none());
    assert!(!prefs.optimize_cost);
    assert!(!prefs.optimize_latency);
}

#[test]
fn test_routing_preferences_structure() {
    let prefs = RoutingPreferences {
        preferred_providers: vec!["openai".to_string(), "anthropic".to_string()],
        excluded_providers: vec!["azure".to_string()],
        strategy_override: Some("least_latency".to_string()),
        tags: vec!["prod".to_string()],
        region: Some("us-east-1".to_string()),
        optimize_cost: true,
        optimize_latency: false,
    };
    assert_eq!(prefs.preferred_providers.len(), 2);
    assert_eq!(prefs.excluded_providers.len(), 1);
    assert!(prefs.optimize_cost);
}

// ==================== CachingPreferences Tests ====================

#[test]
fn test_caching_preferences_default() {
    let prefs = CachingPreferences::default();
    assert!(!prefs.enabled);
    assert!(prefs.ttl_seconds.is_none());
    assert!(prefs.key_prefix.is_none());
    assert!(!prefs.semantic_cache);
    assert!(prefs.similarity_threshold.is_none());
    assert!(prefs.tags.is_empty());
}

#[test]
fn test_caching_preferences_structure() {
    let prefs = CachingPreferences {
        enabled: true,
        ttl_seconds: Some(3600),
        key_prefix: Some("cache:v1:".to_string()),
        semantic_cache: true,
        similarity_threshold: Some(0.95),
        tags: vec!["user:123".to_string()],
    };
    assert!(prefs.enabled);
    assert_eq!(prefs.ttl_seconds, Some(3600));
    assert!(prefs.semantic_cache);
}

// ==================== GatewayRequest Methods Tests ====================

#[test]
fn test_gateway_request_provider_params() {
    let context = RequestContext::new();
    let chat_request = ChatCompletionRequest::default();
    let data = RequestData::ChatCompletion(Box::new(chat_request));
    let mut request = GatewayRequest::new(RequestType::ChatCompletion, data, context);

    request.set_provider_param("temperature", serde_json::json!(0.7));
    request.set_provider_param("max_tokens", serde_json::json!(1000));

    assert_eq!(
        request.get_provider_param("temperature"),
        Some(&serde_json::json!(0.7))
    );
    assert_eq!(
        request.get_provider_param("max_tokens"),
        Some(&serde_json::json!(1000))
    );
    assert!(request.get_provider_param("nonexistent").is_none());
}

#[test]
fn test_gateway_request_with_routing() {
    let context = RequestContext::new();
    let chat_request = ChatCompletionRequest::default();
    let data = RequestData::ChatCompletion(Box::new(chat_request));
    let routing = RoutingPreferences {
        preferred_providers: vec!["openai".to_string()],
        optimize_latency: true,
        ..Default::default()
    };

    let request =
        GatewayRequest::new(RequestType::ChatCompletion, data, context).with_routing(routing);

    assert_eq!(request.routing.preferred_providers.len(), 1);
    assert!(request.routing.optimize_latency);
}

#[test]
fn test_gateway_request_with_caching() {
    let context = RequestContext::new();
    let chat_request = ChatCompletionRequest::default();
    let data = RequestData::ChatCompletion(Box::new(chat_request));
    let caching = CachingPreferences {
        enabled: true,
        semantic_cache: true,
        ..Default::default()
    };

    let request =
        GatewayRequest::new(RequestType::ChatCompletion, data, context).with_caching(caching);

    assert!(request.caching.enabled);
    assert!(request.caching.semantic_cache);
}

#[test]
fn test_gateway_request_add_preferred_provider() {
    let context = RequestContext::new();
    let chat_request = ChatCompletionRequest::default();
    let data = RequestData::ChatCompletion(Box::new(chat_request));

    let request = GatewayRequest::new(RequestType::ChatCompletion, data, context)
        .add_preferred_provider("openai")
        .add_preferred_provider("anthropic");

    assert_eq!(request.routing.preferred_providers.len(), 2);
    assert!(
        request
            .routing
            .preferred_providers
            .contains(&"openai".to_string())
    );
}

#[test]
fn test_gateway_request_exclude_provider() {
    let context = RequestContext::new();
    let chat_request = ChatCompletionRequest::default();
    let data = RequestData::ChatCompletion(Box::new(chat_request));

    let request =
        GatewayRequest::new(RequestType::ChatCompletion, data, context).exclude_provider("azure");

    assert_eq!(request.routing.excluded_providers.len(), 1);
    assert!(
        request
            .routing
            .excluded_providers
            .contains(&"azure".to_string())
    );
}

#[test]
fn test_gateway_request_enable_caching() {
    let context = RequestContext::new();
    let chat_request = ChatCompletionRequest::default();
    let data = RequestData::ChatCompletion(Box::new(chat_request));

    let request =
        GatewayRequest::new(RequestType::ChatCompletion, data, context).enable_caching(Some(7200));

    assert!(request.caching.enabled);
    assert_eq!(request.caching.ttl_seconds, Some(7200));
}

#[test]
fn test_gateway_request_estimated_tokens() {
    let context = RequestContext::new();
    let chat_request = ChatCompletionRequest::default();
    let data = RequestData::ChatCompletion(Box::new(chat_request));
    let request = GatewayRequest::new(RequestType::ChatCompletion, data, context);

    assert!(request.estimated_tokens().is_none());
}

// ==================== CompletionRequest Tests ====================

#[test]
fn test_completion_request_structure() {
    let request = CompletionRequest {
        model: "gpt-3.5-turbo-instruct".to_string(),
        prompt: Some("Complete this:".to_string()),
        max_tokens: Some(100),
        temperature: Some(0.7),
        top_p: Some(0.9),
        n: Some(1),
        stream: Some(false),
        stop: Some(vec!["\n".to_string()]),
        presence_penalty: Some(0.0),
        frequency_penalty: Some(0.0),
        logit_bias: None,
        user: Some("user123".to_string()),
        suffix: None,
        echo: Some(false),
        best_of: Some(1),
        logprobs: None,
    };
    assert_eq!(request.model, "gpt-3.5-turbo-instruct");
    assert_eq!(request.max_tokens, Some(100));
}

// ==================== EmbeddingInput Tests ====================

#[test]
fn test_embedding_input_string() {
    let input = EmbeddingInput::String("Hello world".to_string());
    let json = serde_json::to_value(&input).unwrap();
    assert_eq!(json, "Hello world");
}

#[test]
fn test_embedding_input_array() {
    let input = EmbeddingInput::Array(vec!["Hello".to_string(), "World".to_string()]);
    let json = serde_json::to_value(&input).unwrap();
    assert!(json.is_array());
    assert_eq!(json.as_array().unwrap().len(), 2);
}

// ==================== ModerationInput Tests ====================

#[test]
fn test_moderation_input_string() {
    let input = ModerationInput::String("Check this text".to_string());
    let json = serde_json::to_value(&input).unwrap();
    assert_eq!(json, "Check this text");
}

#[test]
fn test_moderation_input_array() {
    let input = ModerationInput::Array(vec!["Text 1".to_string(), "Text 2".to_string()]);
    let json = serde_json::to_value(&input).unwrap();
    assert!(json.is_array());
}

// ==================== RerankDocument Tests ====================

#[test]
fn test_rerank_document_string() {
    let doc = RerankDocument::String("Document content".to_string());
    let json = serde_json::to_value(&doc).unwrap();
    assert_eq!(json, "Document content");
}

#[test]
fn test_rerank_document_object() {
    let doc = RerankDocument::Object {
        text: "Document with object".to_string(),
    };
    let json = serde_json::to_value(&doc).unwrap();
    assert_eq!(json["text"], "Document with object");
}

// ==================== Model Extraction for Different Types ====================

#[test]
fn test_model_extraction_completion() {
    let context = RequestContext::new();
    let request = CompletionRequest {
        model: "text-davinci-003".to_string(),
        prompt: None,
        max_tokens: None,
        temperature: None,
        top_p: None,
        n: None,
        stream: None,
        stop: None,
        presence_penalty: None,
        frequency_penalty: None,
        logit_bias: None,
        user: None,
        suffix: None,
        echo: None,
        best_of: None,
        logprobs: None,
    };
    let data = RequestData::Completion(request);
    let gateway_request = GatewayRequest::new(RequestType::Completion, data, context);
    assert_eq!(gateway_request.model(), Some("text-davinci-003"));
}

#[test]
fn test_model_extraction_embedding() {
    let context = RequestContext::new();
    let request = EmbeddingRequest {
        model: "text-embedding-ada-002".to_string(),
        input: EmbeddingInput::String("test".to_string()),
        encoding_format: None,
        dimensions: None,
        user: None,
    };
    let data = RequestData::Embedding(request);
    let gateway_request = GatewayRequest::new(RequestType::Embedding, data, context);
    assert_eq!(gateway_request.model(), Some("text-embedding-ada-002"));
}

#[test]
fn test_model_extraction_rerank() {
    let context = RequestContext::new();
    let request = RerankRequest {
        model: "rerank-english-v2.0".to_string(),
        query: "test query".to_string(),
        documents: vec![],
        top_k: None,
        return_documents: None,
        max_chunks_per_doc: None,
    };
    let data = RequestData::Rerank(request);
    let gateway_request = GatewayRequest::new(RequestType::Rerank, data, context);
    assert_eq!(gateway_request.model(), Some("rerank-english-v2.0"));
}

#[test]
fn test_is_streaming_completion() {
    let context = RequestContext::new();
    let request = CompletionRequest {
        model: "model".to_string(),
        stream: Some(true),
        prompt: None,
        max_tokens: None,
        temperature: None,
        top_p: None,
        n: None,
        stop: None,
        presence_penalty: None,
        frequency_penalty: None,
        logit_bias: None,
        user: None,
        suffix: None,
        echo: None,
        best_of: None,
        logprobs: None,
    };
    let data = RequestData::Completion(request);
    let gateway_request = GatewayRequest::new(RequestType::Completion, data, context);
    assert!(gateway_request.is_streaming());
}

#[test]
fn test_is_streaming_embedding() {
    let context = RequestContext::new();
    let request = EmbeddingRequest {
        model: "model".to_string(),
        input: EmbeddingInput::String("test".to_string()),
        encoding_format: None,
        dimensions: None,
        user: None,
    };
    let data = RequestData::Embedding(request);
    let gateway_request = GatewayRequest::new(RequestType::Embedding, data, context);
    assert!(!gateway_request.is_streaming());
}
