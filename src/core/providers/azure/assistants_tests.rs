    use super::*;
    use serde_json::json;

    // ==================== CreateAssistantRequest Tests ====================

    #[test]
    fn test_create_assistant_request_serialization() {
        let request = CreateAssistantRequest {
            model: "gpt-4".to_string(),
            name: Some("My Assistant".to_string()),
            description: Some("A helpful assistant".to_string()),
            instructions: Some("You are helpful".to_string()),
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["model"], "gpt-4");
        assert_eq!(json["name"], "My Assistant");
        assert_eq!(json["description"], "A helpful assistant");
        assert_eq!(json["instructions"], "You are helpful");
    }

    #[test]
    fn test_create_assistant_request_minimal() {
        let request = CreateAssistantRequest {
            model: "gpt-4".to_string(),
            name: None,
            description: None,
            instructions: None,
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["model"], "gpt-4");
        assert!(json["name"].is_null());
    }

    #[test]
    fn test_create_assistant_request_deserialization() {
        let json = json!({
            "model": "gpt-4-turbo",
            "name": "Test",
            "description": null,
            "instructions": "Be helpful"
        });

        let request: CreateAssistantRequest = serde_json::from_value(json).unwrap();
        assert_eq!(request.model, "gpt-4-turbo");
        assert_eq!(request.name, Some("Test".to_string()));
        assert!(request.description.is_none());
        assert_eq!(request.instructions, Some("Be helpful".to_string()));
    }

    // ==================== CreateAssistantResponse Tests ====================

    #[test]
    fn test_create_assistant_response_deserialization() {
        let json = json!({
            "id": "asst_abc123",
            "object": "assistant",
            "created_at": 1699472400
        });

        let response: CreateAssistantResponse = serde_json::from_value(json).unwrap();
        assert_eq!(response.id, "asst_abc123");
        assert_eq!(response.object, "assistant");
        assert_eq!(response.created_at, 1699472400);
    }

    #[test]
    fn test_create_assistant_response_serialization() {
        let response = CreateAssistantResponse {
            id: "asst_test".to_string(),
            object: "assistant".to_string(),
            created_at: 1234567890,
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["id"], "asst_test");
        assert_eq!(json["object"], "assistant");
        assert_eq!(json["created_at"], 1234567890);
    }

    // ==================== ListAssistantsResponse Tests ====================

    #[test]
    fn test_list_assistants_response_empty() {
        let response = ListAssistantsResponse { data: vec![] };

        let json = serde_json::to_value(&response).unwrap();
        assert!(json["data"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_list_assistants_response_with_data() {
        let response = ListAssistantsResponse {
            data: vec![json!({"id": "asst_1"}), json!({"id": "asst_2"})],
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["data"].as_array().unwrap().len(), 2);
    }

    // ==================== RetrieveAssistantResponse Tests ====================

    #[test]
    fn test_retrieve_assistant_response() {
        let response = RetrieveAssistantResponse {
            id: "asst_abc".to_string(),
            object: "assistant".to_string(),
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["id"], "asst_abc");
        assert_eq!(json["object"], "assistant");
    }

    // ==================== ModifyAssistantRequest Tests ====================

    #[test]
    fn test_modify_assistant_request() {
        let request = ModifyAssistantRequest {
            name: Some("New Name".to_string()),
            description: Some("New Description".to_string()),
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["name"], "New Name");
        assert_eq!(json["description"], "New Description");
    }

    #[test]
    fn test_modify_assistant_request_partial() {
        let request = ModifyAssistantRequest {
            name: Some("Only Name".to_string()),
            description: None,
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["name"], "Only Name");
        assert!(json["description"].is_null());
    }

    // ==================== DeleteAssistantResponse Tests ====================

    #[test]
    fn test_delete_assistant_response() {
        let response = DeleteAssistantResponse {
            id: "asst_deleted".to_string(),
            deleted: true,
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["id"], "asst_deleted");
        assert_eq!(json["deleted"], true);
    }

    #[test]
    fn test_delete_assistant_response_not_deleted() {
        let response = DeleteAssistantResponse {
            id: "asst_failed".to_string(),
            deleted: false,
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["deleted"], false);
    }

    // ==================== AssistantApiConfig Tests ====================

    #[test]
    fn test_assistant_api_config_new() {
        let config =
            AssistantApiConfig::new(Some("test-key"), Some("https://api.example.com"), None);

        assert_eq!(config.api_key, Some("test-key".to_string()));
        assert_eq!(config.api_base, Some("https://api.example.com".to_string()));
        assert!(config.headers.is_none());
    }

    #[test]
    fn test_assistant_api_config_with_headers() {
        let mut headers = HashMap::new();
        headers.insert("X-Custom".to_string(), "value".to_string());

        let config = AssistantApiConfig::new(Some("key"), None, Some(headers));

        assert!(config.headers.is_some());
        assert_eq!(
            config.headers.unwrap().get("X-Custom"),
            Some(&"value".to_string())
        );
    }

    #[test]
    fn test_assistant_api_config_empty() {
        let config = AssistantApiConfig::new(None, None, None);

        assert!(config.api_key.is_none());
        assert!(config.api_base.is_none());
        assert!(config.headers.is_none());
    }

    // ==================== Thread Types Tests ====================

    #[test]
    fn test_create_thread_request() {
        let request = CreateThreadRequest {
            messages: Some(vec![json!({"role": "user", "content": "Hello"})]),
        };

        let json = serde_json::to_value(&request).unwrap();
        assert!(json["messages"].is_array());
        assert_eq!(json["messages"][0]["role"], "user");
    }

    #[test]
    fn test_create_thread_request_empty() {
        let request = CreateThreadRequest { messages: None };

        let json = serde_json::to_value(&request).unwrap();
        assert!(json["messages"].is_null());
    }

    #[test]
    fn test_create_thread_response() {
        let response = CreateThreadResponse {
            id: "thread_abc".to_string(),
            object: "thread".to_string(),
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["id"], "thread_abc");
        assert_eq!(json["object"], "thread");
    }

    #[test]
    fn test_retrieve_thread_response() {
        let response = RetrieveThreadResponse {
            id: "thread_xyz".to_string(),
            object: "thread".to_string(),
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["id"], "thread_xyz");
    }

    #[test]
    fn test_modify_thread_request() {
        let mut metadata = HashMap::new();
        metadata.insert("key".to_string(), "value".to_string());

        let request = ModifyThreadRequest {
            metadata: Some(metadata),
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["metadata"]["key"], "value");
    }

    #[test]
    fn test_delete_thread_response() {
        let response = DeleteThreadResponse {
            id: "thread_deleted".to_string(),
            deleted: true,
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["id"], "thread_deleted");
        assert_eq!(json["deleted"], true);
    }

    // ==================== Message Types Tests ====================

    #[test]
    fn test_create_message_request() {
        let request = CreateMessageRequest {
            role: "user".to_string(),
            content: "Hello, assistant!".to_string(),
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["role"], "user");
        assert_eq!(json["content"], "Hello, assistant!");
    }

    #[test]
    fn test_create_message_response() {
        let response = CreateMessageResponse {
            id: "msg_123".to_string(),
            object: "thread.message".to_string(),
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["id"], "msg_123");
        assert_eq!(json["object"], "thread.message");
    }

    #[test]
    fn test_list_messages_response() {
        let response = ListMessagesResponse {
            data: vec![
                json!({"id": "msg_1", "content": "Hi"}),
                json!({"id": "msg_2", "content": "Hello"}),
            ],
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["data"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_retrieve_message_response() {
        let response = RetrieveMessageResponse {
            id: "msg_abc".to_string(),
            object: "thread.message".to_string(),
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["id"], "msg_abc");
    }

    // ==================== Run Types Tests ====================

    #[test]
    fn test_create_run_request() {
        let request = CreateRunRequest {
            assistant_id: "asst_abc".to_string(),
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["assistant_id"], "asst_abc");
    }

    #[test]
    fn test_create_run_response() {
        let response = CreateRunResponse {
            id: "run_123".to_string(),
            object: "thread.run".to_string(),
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["id"], "run_123");
        assert_eq!(json["object"], "thread.run");
    }

    #[test]
    fn test_list_runs_response() {
        let response = ListRunsResponse {
            data: vec![json!({"id": "run_1"}), json!({"id": "run_2"})],
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["data"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_retrieve_run_response() {
        let response = RetrieveRunResponse {
            id: "run_abc".to_string(),
            object: "thread.run".to_string(),
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["id"], "run_abc");
    }

    #[test]
    fn test_submit_tool_outputs_request() {
        let request = SubmitToolOutputsRequest {
            tool_outputs: vec![
                json!({"tool_call_id": "call_1", "output": "result1"}),
                json!({"tool_call_id": "call_2", "output": "result2"}),
            ],
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["tool_outputs"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_submit_tool_outputs_response() {
        let response = SubmitToolOutputsResponse {
            id: "run_123".to_string(),
            object: "thread.run".to_string(),
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["id"], "run_123");
    }

    #[test]
    fn test_cancel_run_response() {
        let response = CancelRunResponse {
            id: "run_cancelled".to_string(),
            object: "thread.run".to_string(),
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["id"], "run_cancelled");
    }

    // ==================== AssistantError Tests ====================

    #[test]
    fn test_assistant_error_authentication() {
        let error = ProviderError::authentication("azure", "Invalid key".to_string());
        let err_str = error.to_string();
        assert!(err_str.contains("azure") || err_str.contains("Invalid key"));
    }

    #[test]
    fn test_assistant_error_request() {
        let error = ProviderError::invalid_request("azure", "Bad request".to_string());
        let err_str = error.to_string();
        assert!(err_str.contains("Bad request") || err_str.contains("invalid"));
    }

    #[test]
    fn test_assistant_error_network() {
        let error = ProviderError::network("azure", "Connection failed".to_string());
        let err_str = error.to_string();
        assert!(err_str.contains("Connection failed") || err_str.contains("network"));
    }

    #[test]
    fn test_assistant_error_configuration() {
        let error = ProviderError::configuration("azure", "Missing config".to_string());
        let err_str = error.to_string();
        assert!(err_str.contains("Missing config") || err_str.contains("configuration"));
    }

    #[test]
    fn test_assistant_error_parsing() {
        let error = ProviderError::serialization("azure", "Invalid JSON".to_string());
        let err_str = error.to_string();
        assert!(err_str.contains("Invalid JSON") || err_str.contains("serialization"));
    }

    #[test]
    fn test_assistant_error_validation() {
        let error = ProviderError::invalid_request("azure", "Invalid model".to_string());
        let err_str = error.to_string();
        assert!(err_str.contains("Invalid model") || err_str.contains("invalid"));
    }

    #[test]
    fn test_assistant_error_api() {
        let error = ProviderError::api_error("azure", 404, "Not found");
        let err_str = error.to_string();
        assert!(err_str.contains("404"));
        assert!(err_str.contains("Not found"));
    }

    // ==================== AzureAssistantUtils Tests ====================

    #[test]
    fn test_get_supported_assistant_models() {
        let models = AzureAssistantUtils::get_supported_assistant_models();
        assert!(models.contains(&"gpt-4"));
        assert!(models.contains(&"gpt-4-turbo"));
        assert!(models.contains(&"gpt-4o"));
        assert!(models.contains(&"gpt-35-turbo"));
        assert!(!models.contains(&"claude-3"));
    }

    #[test]
    fn test_validate_assistant_request_valid() {
        let request = CreateAssistantRequest {
            model: "gpt-4".to_string(),
            name: Some("Test".to_string()),
            description: None,
            instructions: Some("Be helpful".to_string()),
        };

        let result = AzureAssistantUtils::validate_assistant_request(&request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_assistant_request_invalid_model() {
        let request = CreateAssistantRequest {
            model: "unsupported-model".to_string(),
            name: None,
            description: None,
            instructions: None,
        };

        let result = AzureAssistantUtils::validate_assistant_request(&request);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unsupported"));
    }

    #[test]
    fn test_validate_assistant_request_instructions_too_long() {
        let long_instructions = "x".repeat(32769); // Exceeds 32768
        let request = CreateAssistantRequest {
            model: "gpt-4".to_string(),
            name: None,
            description: None,
            instructions: Some(long_instructions),
        };

        let result = AzureAssistantUtils::validate_assistant_request(&request);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("32768"));
    }

    #[test]
    fn test_validate_assistant_request_instructions_at_limit() {
        let instructions = "x".repeat(32768); // Exactly 32768
        let request = CreateAssistantRequest {
            model: "gpt-4".to_string(),
            name: None,
            description: None,
            instructions: Some(instructions),
        };

        let result = AzureAssistantUtils::validate_assistant_request(&request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_assistant_request_all_models() {
        for model in AzureAssistantUtils::get_supported_assistant_models() {
            let request = CreateAssistantRequest {
                model: model.to_string(),
                name: None,
                description: None,
                instructions: None,
            };

            let result = AzureAssistantUtils::validate_assistant_request(&request);
            assert!(result.is_ok(), "Model {} should be valid", model);
        }
    }

    // ==================== AzureAssistantHandler URL Tests ====================

    #[test]
    fn test_azure_assistant_handler_build_assistants_url() {
        let config = AzureConfig::new()
            .with_azure_endpoint("https://myresource.openai.azure.com/".to_string())
            .with_api_version("2024-02-15-preview".to_string())
            .with_api_key("test-key".to_string());

        let handler = AzureAssistantHandler::new(config).unwrap();
        let url = handler.build_assistants_url("");

        assert!(url.contains("myresource.openai.azure.com"));
        assert!(url.contains("openai/assistants"));
        assert!(url.contains("api-version=2024-02-15-preview"));
    }

    #[test]
    fn test_azure_assistant_handler_build_assistants_url_with_id() {
        let config = AzureConfig::new()
            .with_azure_endpoint("https://myresource.openai.azure.com/".to_string())
            .with_api_version("2024-02-15-preview".to_string())
            .with_api_key("test-key".to_string());

        let handler = AzureAssistantHandler::new(config).unwrap();
        let url = handler.build_assistants_url("/asst_abc123");

        assert!(url.contains("/asst_abc123"));
    }

    #[test]
    fn test_azure_assistant_handler_build_threads_url() {
        let config = AzureConfig::new()
            .with_azure_endpoint("https://myresource.openai.azure.com/".to_string())
            .with_api_version("2024-02-15-preview".to_string())
            .with_api_key("test-key".to_string());

        let handler = AzureAssistantHandler::new(config).unwrap();
        let url = handler.build_threads_url("");

        assert!(url.contains("openai/threads"));
        assert!(url.contains("api-version=2024-02-15-preview"));
    }

    #[test]
    fn test_azure_assistant_handler_build_threads_url_with_path() {
        let config = AzureConfig::new()
            .with_azure_endpoint("https://myresource.openai.azure.com/".to_string())
            .with_api_version("2024-02-15-preview".to_string())
            .with_api_key("test-key".to_string());

        let handler = AzureAssistantHandler::new(config).unwrap();
        let url = handler.build_threads_url("/thread_123/messages");

        assert!(url.contains("/thread_123/messages"));
    }

    // ==================== Clone and Debug Tests ====================

    #[test]
    fn test_create_assistant_request_clone() {
        let request = CreateAssistantRequest {
            model: "gpt-4".to_string(),
            name: Some("Test".to_string()),
            description: None,
            instructions: Some("Help".to_string()),
        };

        let cloned = request.clone();
        assert_eq!(request.model, cloned.model);
        assert_eq!(request.name, cloned.name);
    }

    #[test]
    fn test_assistant_api_config_clone() {
        let config = AssistantApiConfig::new(Some("key"), Some("base"), None);

        let cloned = config.clone();
        assert_eq!(config.api_key, cloned.api_key);
        assert_eq!(config.api_base, cloned.api_base);
    }

    #[test]
    fn test_create_assistant_response_debug() {
        let response = CreateAssistantResponse {
            id: "asst_test".to_string(),
            object: "assistant".to_string(),
            created_at: 12345,
        };

        let debug = format!("{:?}", response);
        assert!(debug.contains("CreateAssistantResponse"));
        assert!(debug.contains("asst_test"));
    }
