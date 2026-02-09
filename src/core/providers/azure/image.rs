//! Azure OpenAI Image Generation Handler
//!
//! Complete image generation functionality for Azure OpenAI Service (DALL-E)

use reqwest::header::HeaderMap;
use serde_json::{Value, json};

use crate::core::types::{
    context::RequestContext,
    image::ImageGenerationRequest,
    responses::{ImageData, ImageGenerationResponse},
};

use super::config::AzureConfig;
use super::error::{AzureError, azure_api_error, azure_config_error, azure_header_error};
use super::utils::{AzureEndpointType, AzureUtils};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::provider::ProviderConfig;
use crate::utils::net::http::create_custom_client;

/// Azure OpenAI image generation handler
#[derive(Debug, Clone)]
pub struct AzureImageHandler {
    config: AzureConfig,
    client: reqwest::Client,
}

impl AzureImageHandler {
    /// Create new image generation handler
    pub fn new(config: AzureConfig) -> Result<Self, AzureError> {
        let client = create_custom_client(ProviderConfig::timeout(&config))
            .map_err(|e| azure_config_error(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { config, client })
    }

    /// Build request headers
    async fn build_headers(&self) -> Result<HeaderMap, AzureError> {
        let mut headers = HeaderMap::new();

        // Add API key
        if let Some(api_key) = self.config.get_effective_api_key().await {
            headers.insert(
                "api-key",
                api_key
                    .parse()
                    .map_err(|e| azure_header_error(format!("Invalid API key: {}", e)))?,
            );
        } else {
            return Err(ProviderError::authentication(
                "azure",
                "No API key available",
            ));
        }

        headers.insert(
            "Content-Type",
            "application/json"
                .parse()
                .map_err(|e| azure_header_error(format!("Invalid content type: {}", e)))?,
        );

        // Add custom headers
        for (key, value) in &self.config.custom_headers {
            let header_name = reqwest::header::HeaderName::from_bytes(key.as_bytes())
                .map_err(|e| azure_header_error(format!("Invalid header name: {}", e)))?;
            let header_value = value
                .parse()
                .map_err(|e| azure_header_error(format!("Invalid header value: {}", e)))?;
            headers.insert(header_name, header_value);
        }

        Ok(headers)
    }

    /// Generate image
    pub async fn generate_image(
        &self,
        request: ImageGenerationRequest,
        _context: RequestContext,
    ) -> Result<ImageGenerationResponse, AzureError> {
        // Validate request
        AzureImageUtils::validate_request(&request)?;

        // Get deployment name (for DALL-E models)
        let model_name = request.model.as_deref().unwrap_or("dall-e-3");
        let deployment = self.config.get_effective_deployment_name(model_name);

        // Get Azure endpoint
        let azure_endpoint = self
            .config
            .get_effective_azure_endpoint()
            .ok_or_else(|| azure_config_error("Azure endpoint not configured"))?;

        // Build URL
        let url = AzureUtils::build_azure_url(
            &azure_endpoint,
            &deployment,
            &self.config.api_version,
            AzureEndpointType::Images,
        );

        // Transform request
        let azure_request = AzureImageUtils::transform_request(&request)?;

        // Build headers
        let headers = self.build_headers().await?;

        // Execute request
        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&azure_request)
            .send()
            .await?;

        // Check status
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(azure_api_error(status, error_body));
        }

        // Parse response
        let response_json: Value = response.json().await?;

        // Transform response
        AzureImageUtils::transform_response(response_json)
    }

    /// Edit image (for DALL-E 2)
    pub async fn edit_image(
        &self,
        request: ImageEditRequest,
        _context: RequestContext,
    ) -> Result<ImageGenerationResponse, AzureError> {
        // Get deployment name
        let model_name = request.model.as_str();
        let deployment = self.config.get_effective_deployment_name(model_name);

        // Get Azure endpoint
        let azure_endpoint = self
            .config
            .get_effective_azure_endpoint()
            .ok_or_else(|| azure_config_error("Azure endpoint not configured"))?;

        // Build URL
        let url = AzureUtils::build_azure_url(
            &azure_endpoint,
            &deployment,
            &self.config.api_version,
            AzureEndpointType::ImageEdits,
        );

        // Build multipart form for image edit
        let mut form = reqwest::multipart::Form::new()
            .text("prompt", request.prompt)
            .part("image", request.image)
            .text("n", request.n.unwrap_or(1).to_string());

        if let Some(size) = request.size {
            form = form.text("size", size);
        }

        if let Some(mask) = request.mask {
            form = form.part("mask", mask);
        }

        // Build headers
        let mut headers = self.build_headers().await?;
        headers.remove("Content-Type"); // Let reqwest set multipart content type

        // Execute request
        let response = self
            .client
            .post(&url)
            .headers(headers)
            .multipart(form)
            .send()
            .await?;

        // Check status
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(azure_api_error(status, error_body));
        }

        // Parse response
        let response_json: Value = response.json().await?;

        // Transform response
        AzureImageUtils::transform_response(response_json)
    }
}

/// Image edit request
#[derive(Debug)]
pub struct ImageEditRequest {
    pub model: String,
    pub image: reqwest::multipart::Part,
    pub mask: Option<reqwest::multipart::Part>,
    pub prompt: String,
    pub n: Option<usize>,
    pub size: Option<String>,
}

/// Azure image generation utilities
pub struct AzureImageUtils;

impl AzureImageUtils {
    /// Validate image generation request
    pub fn validate_request(request: &ImageGenerationRequest) -> Result<(), AzureError> {
        if request.prompt.is_empty() {
            return Err(azure_config_error("Prompt cannot be empty"));
        }

        // Validate size if specified
        if let Some(size) = &request.size {
            let model_name = request.model.as_deref().unwrap_or("dall-e-3");
            if !Self::is_valid_size(size, model_name) {
                return Err(azure_config_error(format!(
                    "Invalid size '{}' for model '{}'",
                    size, model_name
                )));
            }
        }

        // Validate quality
        if let Some(quality) = &request.quality {
            if !["standard", "hd"].contains(&quality.as_str()) {
                return Err(azure_config_error(format!(
                    "Invalid quality '{}'. Must be 'standard' or 'hd'",
                    quality
                )));
            }
        }

        // Validate style
        if let Some(style) = &request.style {
            if !["vivid", "natural"].contains(&style.as_str()) {
                return Err(azure_config_error(format!(
                    "Invalid style '{}'. Must be 'vivid' or 'natural'",
                    style
                )));
            }
        }

        // Validate n (number of images)
        if let Some(n) = request.n {
            if n == 0 || n > 10 {
                return Err(azure_config_error(
                    "Number of images must be between 1 and 10",
                ));
            }
        }

        Ok(())
    }

    /// Transform request to Azure format
    pub fn transform_request(request: &ImageGenerationRequest) -> Result<Value, AzureError> {
        let mut body = json!({
            "prompt": request.prompt,
        });

        // Add optional parameters
        if let Some(n) = request.n {
            body["n"] = json!(n);
        }

        if let Some(size) = &request.size {
            body["size"] = json!(size);
        }

        if let Some(quality) = &request.quality {
            body["quality"] = json!(quality);
        }

        if let Some(style) = &request.style {
            body["style"] = json!(style);
        }

        if let Some(response_format) = &request.response_format {
            body["response_format"] = json!(response_format);
        }

        if let Some(user) = &request.user {
            body["user"] = json!(user);
        }

        Ok(body)
    }

    /// Transform Azure response to standard format
    pub fn transform_response(response: Value) -> Result<ImageGenerationResponse, AzureError> {
        let created = response["created"].as_u64().unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        });

        let data = response["data"]
            .as_array()
            .ok_or_else(|| ProviderError::serialization("azure", "Missing data array"))?
            .iter()
            .map(|item| {
                // Handle both URL and base64 responses
                if let Some(url) = item["url"].as_str() {
                    ImageData {
                        url: Some(url.to_string()),
                        b64_json: None,
                        revised_prompt: item["revised_prompt"].as_str().map(|s| s.to_string()),
                    }
                } else if let Some(b64) = item["b64_json"].as_str() {
                    ImageData {
                        url: None,
                        b64_json: Some(b64.to_string()),
                        revised_prompt: item["revised_prompt"].as_str().map(|s| s.to_string()),
                    }
                } else {
                    ImageData {
                        url: None,
                        b64_json: None,
                        revised_prompt: None,
                    }
                }
            })
            .collect();

        Ok(ImageGenerationResponse { created, data })
    }

    /// Check if size is valid for model
    fn is_valid_size(size: &str, model: &str) -> bool {
        let lower_model = model.to_lowercase();

        if lower_model.contains("dall-e-3") {
            // DALL-E 3 supported sizes
            matches!(size, "1024x1024" | "1024x1792" | "1792x1024")
        } else if lower_model.contains("dall-e-2") {
            // DALL-E 2 supported sizes
            matches!(size, "256x256" | "512x512" | "1024x1024")
        } else {
            // Default to true for unknown models
            true
        }
    }
}

#[cfg(test)]
mod image_tests;
