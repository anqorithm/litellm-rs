//! Image Generation Module for Bedrock
//!
//! Handles Stability AI and Amazon Nova Canvas image generation

use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::image::ImageGenerationRequest;
use crate::core::types::responses::{ImageData, ImageGenerationResponse};
use serde::{Deserialize, Serialize};

/// Stability AI text to image request
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StabilityTextToImageRequest {
    pub text_prompts: Vec<TextPrompt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cfg_scale: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub samples: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steps: Option<u32>,
}

/// Text prompt for Stability AI
#[derive(Debug, Serialize)]
pub struct TextPrompt {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f32>,
}

/// Stability AI response
#[derive(Debug, Deserialize)]
pub struct StabilityResponse {
    pub result: String,
    pub artifacts: Vec<StabilityArtifact>,
}

/// Stability AI artifact
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StabilityArtifact {
    pub base64: String,
    pub finish_reason: String,
    pub seed: i64,
}

/// Nova Canvas request
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NovaCanvasRequest {
    pub text_to_image_params: TextToImageParams,
    pub task_type: String, // "TEXT_IMAGE"
}

/// Text to image parameters for Nova Canvas
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextToImageParams {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negative_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cfg_scale: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_of_images: Option<u32>,
}

/// Nova Canvas response
#[derive(Debug, Deserialize)]
pub struct NovaCanvasResponse {
    pub images: Vec<NovaImage>,
}

/// Nova image
#[derive(Debug, Deserialize)]
pub struct NovaImage {
    pub base64: String,
}

/// Execute image generation request
pub async fn execute_image_generation(
    client: &crate::core::providers::bedrock::client::BedrockClient,
    request: &ImageGenerationRequest,
) -> Result<ImageGenerationResponse, ProviderError> {
    let default_model = "dall-e-3".to_string();
    let model = request.model.as_ref().unwrap_or(&default_model);

    if model.contains("stability") {
        execute_stability_generation(client, request, model).await
    } else if model.contains("nova-canvas") {
        execute_nova_generation(client, request, model).await
    } else {
        Err(ProviderError::model_not_found(
            "bedrock",
            format!("Image generation model {} not supported", model),
        ))
    }
}

/// Execute Stability AI image generation
async fn execute_stability_generation(
    client: &crate::core::providers::bedrock::client::BedrockClient,
    request: &ImageGenerationRequest,
    model: &str,
) -> Result<ImageGenerationResponse, ProviderError> {
    let stability_request = StabilityTextToImageRequest {
        text_prompts: vec![TextPrompt {
            text: request.prompt.clone(),
            weight: Some(1.0),
        }],
        cfg_scale: Some(7.0),
        height: request
            .size
            .as_ref()
            .and_then(|s| s.split('x').next().and_then(|h| h.parse().ok())),
        width: request
            .size
            .as_ref()
            .and_then(|s| s.split('x').nth(1).and_then(|w| w.parse().ok())),
        samples: request.n,
        seed: None,
        steps: request.quality.as_ref().and_then(|q| match q.as_str() {
            "standard" => Some(30),
            "hd" => Some(50),
            _ => None,
        }),
    };

    let body = serde_json::to_value(stability_request)?;
    let response = client.send_request(model, "invoke", &body).await?;
    let stability_response: StabilityResponse = response
        .json()
        .await
        .map_err(|e| ProviderError::response_parsing("bedrock", e.to_string()))?;

    // Convert to OpenAI format
    let data: Vec<ImageData> = stability_response
        .artifacts
        .into_iter()
        .map(|artifact| ImageData {
            url: None,
            b64_json: Some(artifact.base64),
            revised_prompt: None,
        })
        .collect();

    Ok(ImageGenerationResponse {
        created: chrono::Utc::now().timestamp() as u64,
        data,
    })
}

/// Execute Nova Canvas image generation
async fn execute_nova_generation(
    client: &crate::core::providers::bedrock::client::BedrockClient,
    request: &ImageGenerationRequest,
    model: &str,
) -> Result<ImageGenerationResponse, ProviderError> {
    let nova_request = NovaCanvasRequest {
        text_to_image_params: TextToImageParams {
            text: request.prompt.clone(),
            negative_text: None,
            height: request
                .size
                .as_ref()
                .and_then(|s| s.split('x').next().and_then(|h| h.parse().ok())),
            width: request
                .size
                .as_ref()
                .and_then(|s| s.split('x').nth(1).and_then(|w| w.parse().ok())),
            cfg_scale: Some(7.0),
            seed: None,
            number_of_images: request.n,
        },
        task_type: "TEXT_IMAGE".to_string(),
    };

    let body = serde_json::to_value(nova_request)?;
    let response = client.send_request(model, "invoke", &body).await?;
    let nova_response: NovaCanvasResponse = response
        .json()
        .await
        .map_err(|e| ProviderError::response_parsing("bedrock", e.to_string()))?;

    // Convert to OpenAI format
    let data: Vec<ImageData> = nova_response
        .images
        .into_iter()
        .map(|image| ImageData {
            url: None,
            b64_json: Some(image.base64),
            revised_prompt: None,
        })
        .collect();

    Ok(ImageGenerationResponse {
        created: chrono::Utc::now().timestamp() as u64,
        data,
    })
}
