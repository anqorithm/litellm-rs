use super::*;

fn create_test_request() -> ImageGenerationRequest {
    ImageGenerationRequest {
        prompt: "A beautiful sunset over the ocean".to_string(),
        model: Some("dall-e-3".to_string()),
        n: Some(1),
        size: Some("1024x1024".to_string()),
        quality: Some("standard".to_string()),
        style: Some("vivid".to_string()),
        response_format: None,
        user: None,
    }
}

#[test]
fn test_validate_request_valid() {
    let request = create_test_request();
    assert!(AzureImageUtils::validate_request(&request).is_ok());
}

#[test]
fn test_validate_request_empty_prompt() {
    let request = ImageGenerationRequest {
        prompt: "".to_string(),
        model: Some("dall-e-3".to_string()),
        n: None,
        size: None,
        quality: None,
        response_format: None,
        style: None,
        user: None,
    };
    let result = AzureImageUtils::validate_request(&request);
    assert!(result.is_err());
}

#[test]
fn test_validate_request_invalid_size_dalle3() {
    let request = ImageGenerationRequest {
        prompt: "Test".to_string(),
        model: Some("dall-e-3".to_string()),
        size: Some("256x256".to_string()), // Not valid for DALL-E 3
        n: None,
        quality: None,
        response_format: None,
        style: None,
        user: None,
    };
    let result = AzureImageUtils::validate_request(&request);
    assert!(result.is_err());
}

#[test]
fn test_validate_request_invalid_size_dalle2() {
    let request = ImageGenerationRequest {
        prompt: "Test".to_string(),
        model: Some("dall-e-2".to_string()),
        size: Some("1792x1024".to_string()), // Not valid for DALL-E 2
        n: None,
        quality: None,
        response_format: None,
        style: None,
        user: None,
    };
    let result = AzureImageUtils::validate_request(&request);
    assert!(result.is_err());
}

#[test]
fn test_validate_request_invalid_quality() {
    let request = ImageGenerationRequest {
        prompt: "Test".to_string(),
        model: Some("dall-e-3".to_string()),
        quality: Some("ultra".to_string()), // Invalid
        n: None,
        size: None,
        response_format: None,
        style: None,
        user: None,
    };
    let result = AzureImageUtils::validate_request(&request);
    assert!(result.is_err());
}

#[test]
fn test_validate_request_invalid_style() {
    let request = ImageGenerationRequest {
        prompt: "Test".to_string(),
        model: Some("dall-e-3".to_string()),
        style: Some("artistic".to_string()), // Invalid
        n: None,
        size: None,
        quality: None,
        response_format: None,
        user: None,
    };
    let result = AzureImageUtils::validate_request(&request);
    assert!(result.is_err());
}

#[test]
fn test_validate_request_n_zero() {
    let request = ImageGenerationRequest {
        prompt: "Test".to_string(),
        model: Some("dall-e-3".to_string()),
        n: Some(0),
        size: None,
        quality: None,
        response_format: None,
        style: None,
        user: None,
    };
    let result = AzureImageUtils::validate_request(&request);
    assert!(result.is_err());
}

#[test]
fn test_validate_request_n_too_large() {
    let request = ImageGenerationRequest {
        prompt: "Test".to_string(),
        model: Some("dall-e-3".to_string()),
        n: Some(11),
        size: None,
        quality: None,
        response_format: None,
        style: None,
        user: None,
    };
    let result = AzureImageUtils::validate_request(&request);
    assert!(result.is_err());
}

#[test]
fn test_is_valid_size_dalle3() {
    assert!(AzureImageUtils::is_valid_size("1024x1024", "dall-e-3"));
    assert!(AzureImageUtils::is_valid_size("1024x1792", "dall-e-3"));
    assert!(AzureImageUtils::is_valid_size("1792x1024", "dall-e-3"));
    assert!(!AzureImageUtils::is_valid_size("256x256", "dall-e-3"));
    assert!(!AzureImageUtils::is_valid_size("512x512", "dall-e-3"));
}

#[test]
fn test_is_valid_size_dalle2() {
    assert!(AzureImageUtils::is_valid_size("256x256", "dall-e-2"));
    assert!(AzureImageUtils::is_valid_size("512x512", "dall-e-2"));
    assert!(AzureImageUtils::is_valid_size("1024x1024", "dall-e-2"));
    assert!(!AzureImageUtils::is_valid_size("1024x1792", "dall-e-2"));
    assert!(!AzureImageUtils::is_valid_size("1792x1024", "dall-e-2"));
}

#[test]
fn test_is_valid_size_unknown_model() {
    // Unknown models should accept any size
    assert!(AzureImageUtils::is_valid_size("1024x1024", "unknown-model"));
    assert!(AzureImageUtils::is_valid_size("4096x4096", "future-dalle"));
}

#[test]
fn test_transform_request_basic() {
    let request = ImageGenerationRequest {
        prompt: "A cat".to_string(),
        model: Some("dall-e-3".to_string()),
        n: None,
        size: None,
        quality: None,
        response_format: None,
        style: None,
        user: None,
    };

    let result = AzureImageUtils::transform_request(&request);
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value["prompt"], "A cat");
}

#[test]
fn test_transform_request_with_options() {
    let request = create_test_request();

    let result = AzureImageUtils::transform_request(&request);
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value["prompt"], "A beautiful sunset over the ocean");
    assert_eq!(value["n"], 1);
    assert_eq!(value["size"], "1024x1024");
    assert_eq!(value["quality"], "standard");
    assert_eq!(value["style"], "vivid");
}

#[test]
fn test_transform_request_with_user() {
    let request = ImageGenerationRequest {
        prompt: "Test".to_string(),
        model: Some("dall-e-3".to_string()),
        user: Some("user-123".to_string()),
        n: None,
        size: None,
        quality: None,
        response_format: None,
        style: None,
    };

    let result = AzureImageUtils::transform_request(&request);
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value["user"], "user-123");
}

#[test]
fn test_transform_response_url() {
    let response = json!({
        "created": 1234567890,
        "data": [{
            "url": "https://example.com/image.png",
            "revised_prompt": "A revised prompt"
        }]
    });

    let result = AzureImageUtils::transform_response(response);
    assert!(result.is_ok());
    let image_response = result.unwrap();
    assert_eq!(image_response.created, 1234567890);
    assert_eq!(image_response.data.len(), 1);
    assert_eq!(
        image_response.data[0].url,
        Some("https://example.com/image.png".to_string())
    );
    assert_eq!(
        image_response.data[0].revised_prompt,
        Some("A revised prompt".to_string())
    );
}

#[test]
fn test_transform_response_b64() {
    let response = json!({
        "created": 1234567890,
        "data": [{
            "b64_json": "base64encodeddata",
            "revised_prompt": "Another prompt"
        }]
    });

    let result = AzureImageUtils::transform_response(response);
    assert!(result.is_ok());
    let image_response = result.unwrap();
    assert_eq!(image_response.data.len(), 1);
    assert_eq!(
        image_response.data[0].b64_json,
        Some("base64encodeddata".to_string())
    );
    assert!(image_response.data[0].url.is_none());
}

#[test]
fn test_transform_response_multiple_images() {
    let response = json!({
        "created": 1234567890,
        "data": [
            {"url": "https://example.com/image1.png"},
            {"url": "https://example.com/image2.png"},
            {"url": "https://example.com/image3.png"}
        ]
    });

    let result = AzureImageUtils::transform_response(response);
    assert!(result.is_ok());
    let image_response = result.unwrap();
    assert_eq!(image_response.data.len(), 3);
}

#[test]
fn test_transform_response_missing_data() {
    let response = json!({
        "created": 1234567890
    });

    let result = AzureImageUtils::transform_response(response);
    assert!(result.is_err());
}

#[test]
fn test_azure_image_handler_new() {
    let config =
        AzureConfig::new().with_azure_endpoint("https://test.openai.azure.com".to_string());
    let handler = AzureImageHandler::new(config);
    assert!(handler.is_ok());
}

#[test]
fn test_validate_request_valid_quality_hd() {
    let request = ImageGenerationRequest {
        prompt: "Test".to_string(),
        model: Some("dall-e-3".to_string()),
        quality: Some("hd".to_string()),
        n: None,
        size: None,
        response_format: None,
        style: None,
        user: None,
    };
    assert!(AzureImageUtils::validate_request(&request).is_ok());
}

#[test]
fn test_validate_request_valid_style_natural() {
    let request = ImageGenerationRequest {
        prompt: "Test".to_string(),
        model: Some("dall-e-3".to_string()),
        style: Some("natural".to_string()),
        n: None,
        size: None,
        quality: None,
        response_format: None,
        user: None,
    };
    assert!(AzureImageUtils::validate_request(&request).is_ok());
}

#[test]
fn test_validate_request_n_boundary() {
    // n=1 should be valid
    let request = ImageGenerationRequest {
        prompt: "Test".to_string(),
        model: Some("dall-e-3".to_string()),
        n: Some(1),
        size: None,
        quality: None,
        response_format: None,
        style: None,
        user: None,
    };
    assert!(AzureImageUtils::validate_request(&request).is_ok());

    // n=10 should be valid
    let request = ImageGenerationRequest {
        prompt: "Test".to_string(),
        model: Some("dall-e-3".to_string()),
        n: Some(10),
        size: None,
        quality: None,
        response_format: None,
        style: None,
        user: None,
    };
    assert!(AzureImageUtils::validate_request(&request).is_ok());
}

#[test]
fn test_transform_request_response_format() {
    let request = ImageGenerationRequest {
        prompt: "Test".to_string(),
        model: Some("dall-e-3".to_string()),
        response_format: Some("b64_json".to_string()),
        n: None,
        size: None,
        quality: None,
        style: None,
        user: None,
    };

    let result = AzureImageUtils::transform_request(&request);
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value["response_format"], "b64_json");
}
