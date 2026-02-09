use super::*;

#[test]
fn test_get_model_info_valid() {
    let info = get_model_info("meta-llama/Llama-3.3-70B-Instruct-Turbo");
    assert!(info.is_some());
    let info = info.unwrap();
    assert_eq!(info.model_id, "meta-llama/Llama-3.3-70B-Instruct-Turbo");
    assert_eq!(info.display_name, "Llama 3.3 70B Instruct Turbo");
    assert_eq!(info.max_context_length, 131072);
    assert!(info.supports_tools);
    assert!(!info.supports_multimodal);
}

#[test]
fn test_get_model_info_invalid() {
    let info = get_model_info("nonexistent-model");
    assert!(info.is_none());
}

#[test]
fn test_is_function_calling_model() {
    assert!(is_function_calling_model(
        "meta-llama/Llama-3.3-70B-Instruct-Turbo"
    ));
    assert!(is_function_calling_model("deepseek-ai/DeepSeek-V3"));
    assert!(!is_function_calling_model(
        "meta-llama/Llama-3.2-90B-Vision-Instruct-Turbo"
    ));
    assert!(!is_function_calling_model("nonexistent-model"));
}

#[test]
fn test_is_vision_model() {
    assert!(is_vision_model(
        "meta-llama/Llama-3.2-90B-Vision-Instruct-Turbo"
    ));
    assert!(is_vision_model(
        "meta-llama/Llama-3.2-11B-Vision-Instruct-Turbo"
    ));
    assert!(!is_vision_model("meta-llama/Llama-3.3-70B-Instruct-Turbo"));
    assert!(!is_vision_model("nonexistent-model"));
}

#[test]
fn test_is_embedding_model() {
    assert!(is_embedding_model(
        "togethercomputer/m2-bert-80M-2k-retrieval"
    ));
    assert!(is_embedding_model("BAAI/bge-large-en-v1.5"));
    assert!(!is_embedding_model(
        "meta-llama/Llama-3.3-70B-Instruct-Turbo"
    ));
    assert!(!is_embedding_model("nonexistent-model"));
}

#[test]
fn test_is_rerank_model() {
    assert!(is_rerank_model("Salesforce/Llama-Rank-V1"));
    assert!(!is_rerank_model("meta-llama/Llama-3.3-70B-Instruct-Turbo"));
    assert!(!is_rerank_model("nonexistent-model"));
}

#[test]
fn test_get_available_models() {
    let models = get_available_models();
    assert!(!models.is_empty());
    assert!(models.contains(&"meta-llama/Llama-3.3-70B-Instruct-Turbo"));
    assert!(models.contains(&"deepseek-ai/DeepSeek-V3"));
}

#[test]
fn test_get_tool_capable_models() {
    let models = get_tool_capable_models();
    assert!(!models.is_empty());
    assert!(models.contains(&"meta-llama/Llama-3.3-70B-Instruct-Turbo"));
    // Embedding models don't support tools
    assert!(!models.contains(&"togethercomputer/m2-bert-80M-2k-retrieval"));
}

#[test]
fn test_get_embedding_models() {
    let models = get_embedding_models();
    assert!(!models.is_empty());
    assert!(models.contains(&"togethercomputer/m2-bert-80M-2k-retrieval"));
    assert!(models.contains(&"BAAI/bge-large-en-v1.5"));
}

#[test]
fn test_get_rerank_models() {
    let models = get_rerank_models();
    assert!(!models.is_empty());
    assert!(models.contains(&"Salesforce/Llama-Rank-V1"));
}

#[test]
fn test_model_info_costs() {
    let info = get_model_info("meta-llama/Meta-Llama-3.1-8B-Instruct-Turbo").unwrap();
    assert!(info.input_cost_per_million > 0.0);
    assert!(info.output_cost_per_million > 0.0);
}

#[test]
fn test_together_model_enum() {
    let model = TogetherModel::Llama3_3_70B_Instruct_Turbo;
    assert_eq!(format!("{:?}", model), "Llama3_3_70B_Instruct_Turbo");

    let model = TogetherModel::DeepSeekV3;
    assert_eq!(format!("{:?}", model), "DeepSeekV3");
}

#[test]
fn test_deepseek_models() {
    let v3 = get_model_info("deepseek-ai/DeepSeek-V3").unwrap();
    assert!(v3.supports_tools);
    assert!(!v3.supports_multimodal);
    assert_eq!(v3.max_context_length, 131072);

    let r1 = get_model_info("deepseek-ai/DeepSeek-R1").unwrap();
    assert!(r1.supports_tools);
}

#[test]
fn test_qwen_models() {
    let qwen = get_model_info("Qwen/Qwen2.5-72B-Instruct-Turbo").unwrap();
    assert_eq!(qwen.display_name, "Qwen 2.5 72B Instruct Turbo");
    assert!(qwen.supports_tools);
}

#[test]
fn test_mistral_models() {
    let mixtral = get_model_info("mistralai/Mixtral-8x22B-Instruct-v0.1").unwrap();
    assert!(mixtral.supports_tools);
    assert_eq!(mixtral.max_context_length, 65536);
}

#[test]
fn test_pricing_category() {
    assert_eq!(
        get_pricing_category("model-3b"),
        Some("together-ai-up-to-4b")
    );
    assert_eq!(
        get_pricing_category("model-7b"),
        Some("together-ai-4.1b-8b")
    );
    assert_eq!(
        get_pricing_category("model-13b"),
        Some("together-ai-8.1b-21b")
    );
    assert_eq!(
        get_pricing_category("model-34b"),
        Some("together-ai-21.1b-41b")
    );
    assert_eq!(
        get_pricing_category("model-70b"),
        Some("together-ai-41.1b-80b")
    );
    assert_eq!(
        get_pricing_category("model-100b"),
        Some("together-ai-81.1b-110b")
    );
    assert_eq!(get_pricing_category("model-unknown"), None);
}
