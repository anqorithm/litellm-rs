//! Runway ML Provider Implementation
//!
//! Main provider implementation for Runway ML video and image generation.

use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::core::providers::base::{GlobalPoolManager, HeaderPair, HttpMethod, header};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::{
    error_mapper::trait_def::ErrorMapper, provider::ProviderConfig,
    provider::llm_provider::trait_definition::LLMProvider,
};
use crate::core::types::{
    image::ImageGenerationRequest,
    model::ModelInfo,
    responses::{ImageData, ImageGenerationResponse},
};

use super::{RunwayMLConfig, RunwayMLErrorMapper, get_runwayml_registry};

const PROVIDER_NAME: &str = "runwayml";

/// Runway ML task status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TaskStatus {
    /// Task is pending
    Pending,
    /// Task is in the queue
    Throttled,
    /// Task is running
    Running,
    /// Task completed successfully
    Succeeded,
    /// Task failed
    Failed,
    /// Task was cancelled
    Cancelled,
}

/// Runway ML task request
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskRequest {
    /// The model to use (e.g., "gen3a_turbo")
    pub model: String,
    /// Text prompt for generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_text: Option<String>,
    /// Image URL for image-to-video
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_image: Option<String>,
    /// Video duration in seconds (5 or 10)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
    /// Aspect ratio
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ratio: Option<String>,
    /// Seed for reproducibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    /// Whether to watermark the output
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watermark: Option<bool>,
}

/// Runway ML task response
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskResponse {
    /// Task ID
    pub id: String,
    /// Task status
    pub status: TaskStatus,
    /// Creation timestamp
    #[serde(default)]
    pub created_at: Option<String>,
    /// Output URLs (available when succeeded)
    #[serde(default)]
    pub output: Option<Vec<String>>,
    /// Error message (if failed)
    #[serde(default)]
    pub failure: Option<String>,
    /// Failure code
    #[serde(default)]
    pub failure_code: Option<String>,
    /// Progress percentage
    #[serde(default)]
    pub progress: Option<f32>,
}

/// Runway ML video generation response
#[derive(Debug, Clone)]
pub struct VideoGenerationResponse {
    /// Task ID
    pub task_id: String,
    /// Video URLs
    pub video_urls: Vec<String>,
    /// Generation duration
    pub duration_seconds: u32,
}

/// Runway ML provider implementation
#[derive(Debug, Clone)]
pub struct RunwayMLProvider {
    config: RunwayMLConfig,
    pool_manager: Arc<GlobalPoolManager>,
    supported_models: Vec<ModelInfo>,
}

impl RunwayMLProvider {
    /// Create a new Runway ML provider
    pub fn new(config: RunwayMLConfig) -> Result<Self, ProviderError> {
        config
            .validate()
            .map_err(|e| ProviderError::configuration(PROVIDER_NAME, e))?;

        let pool_manager = Arc::new(
            GlobalPoolManager::new()
                .map_err(|e| ProviderError::configuration(PROVIDER_NAME, e.to_string()))?,
        );

        let supported_models = get_runwayml_registry().models().to_vec();

        Ok(Self {
            config,
            pool_manager,
            supported_models,
        })
    }

    /// Create provider with API key
    pub async fn with_api_key(api_key: impl Into<String>) -> Result<Self, ProviderError> {
        let config = RunwayMLConfig::new(api_key);
        Self::new(config)
    }

    /// Create provider from environment
    pub fn from_env() -> Result<Self, ProviderError> {
        let config = RunwayMLConfig::from_env();
        Self::new(config)
    }

    /// Generate headers for Runway ML API requests
    fn get_request_headers(&self) -> Vec<HeaderPair> {
        let mut headers = Vec::with_capacity(3);

        if let Some(api_key) = &self.config.base.api_key {
            headers.push(header("Authorization", format!("Bearer {}", api_key)));
        }

        headers.push(header("Content-Type", "application/json".to_string()));
        headers.push(header("Accept", "application/json".to_string()));

        // Add API version header if specified
        if let Some(api_version) = &self.config.base.api_version {
            headers.push(header("X-Runway-Version", api_version.clone()));
        }

        headers
    }

    /// Create a video generation task
    pub async fn create_video_task(
        &self,
        prompt_text: Option<String>,
        prompt_image: Option<String>,
        model: Option<&str>,
        duration: Option<u32>,
        ratio: Option<String>,
        seed: Option<u64>,
    ) -> Result<TaskResponse, ProviderError> {
        let api_model = model
            .map(|m| get_runwayml_registry().get_api_model(m))
            .unwrap_or("gen3a_turbo");

        let request = CreateTaskRequest {
            model: api_model.to_string(),
            prompt_text,
            prompt_image,
            duration: duration.or(Some(self.config.default_video_duration)),
            ratio,
            seed,
            watermark: Some(self.config.watermark),
        };

        self.submit_task(&request).await
    }

    /// Submit a task to Runway ML
    async fn submit_task(
        &self,
        request: &CreateTaskRequest,
    ) -> Result<TaskResponse, ProviderError> {
        let url = self.config.get_generate_url();
        let headers = self.get_request_headers();
        let body = serde_json::to_value(request)
            .map_err(|e| ProviderError::serialization(PROVIDER_NAME, e.to_string()))?;

        let response = self
            .pool_manager
            .execute_request(&url, HttpMethod::POST, headers, Some(body))
            .await?;

        let status = response.status();
        let response_bytes = response
            .bytes()
            .await
            .map_err(|e| ProviderError::network(PROVIDER_NAME, e.to_string()))?;

        if !status.is_success() {
            let error_text = String::from_utf8_lossy(&response_bytes);
            let mapper = RunwayMLErrorMapper;
            return Err(mapper.map_http_error(status.as_u16(), &error_text));
        }

        serde_json::from_slice(&response_bytes)
            .map_err(|e| ProviderError::response_parsing(PROVIDER_NAME, e.to_string()))
    }

    /// Get task status
    async fn get_task(&self, task_id: &str) -> Result<TaskResponse, ProviderError> {
        let url = self.config.get_task_url(task_id);
        let headers = self.get_request_headers();

        let response = self
            .pool_manager
            .execute_request(&url, HttpMethod::GET, headers, None)
            .await?;

        let status = response.status();
        let response_bytes = response
            .bytes()
            .await
            .map_err(|e| ProviderError::network(PROVIDER_NAME, e.to_string()))?;

        if !status.is_success() {
            let error_text = String::from_utf8_lossy(&response_bytes);
            let mapper = RunwayMLErrorMapper;
            return Err(mapper.map_http_error(status.as_u16(), &error_text));
        }

        serde_json::from_slice(&response_bytes)
            .map_err(|e| ProviderError::response_parsing(PROVIDER_NAME, e.to_string()))
    }

    /// Poll task until completion
    async fn poll_task(&self, task_id: &str) -> Result<TaskResponse, ProviderError> {
        let polling_delay = std::time::Duration::from_secs(self.config.polling_delay_seconds);

        for _ in 0..self.config.polling_retries {
            tokio::time::sleep(polling_delay).await;

            let task = self.get_task(task_id).await?;

            match task.status {
                TaskStatus::Succeeded => return Ok(task),
                TaskStatus::Failed => {
                    let error_msg = task.failure.unwrap_or_else(|| "Task failed".to_string());
                    return Err(ProviderError::api_error(
                        PROVIDER_NAME,
                        500,
                        format!("Video generation failed: {}", error_msg),
                    ));
                }
                TaskStatus::Cancelled => {
                    return Err(ProviderError::cancelled(
                        PROVIDER_NAME,
                        "video_generation",
                        Some("Task was cancelled".to_string()),
                    ));
                }
                _ => {
                    // Still processing, continue polling
                }
            }
        }

        Err(ProviderError::timeout(
            PROVIDER_NAME,
            "Maximum retries exceeded waiting for video generation",
        ))
    }

    /// Create video task and wait for completion
    pub async fn generate_video(
        &self,
        prompt_text: Option<String>,
        prompt_image: Option<String>,
        model: Option<&str>,
        duration: Option<u32>,
        ratio: Option<String>,
        seed: Option<u64>,
    ) -> Result<VideoGenerationResponse, ProviderError> {
        // Create the task
        let task = self
            .create_video_task(prompt_text, prompt_image, model, duration, ratio, seed)
            .await?;

        // Poll until completion
        let completed_task = self.poll_task(&task.id).await?;

        // Extract video URLs
        let video_urls = completed_task.output.unwrap_or_default();

        Ok(VideoGenerationResponse {
            task_id: completed_task.id,
            video_urls,
            duration_seconds: duration.unwrap_or(self.config.default_video_duration),
        })
    }

    /// Transform image generation request to video generation
    fn transform_image_to_video_request(
        &self,
        request: &ImageGenerationRequest,
    ) -> CreateTaskRequest {
        let registry = get_runwayml_registry();
        let model = request.model.as_deref().unwrap_or("gen3a_turbo");
        let api_model = registry.get_api_model(model);

        // Map size to aspect ratio
        let ratio = request.size.as_ref().map(|size| {
            match size.as_str() {
                "1024x1024" | "512x512" => "1:1",
                "1792x1024" | "1280x720" => "16:9",
                "1024x1792" | "720x1280" => "9:16",
                "1280x768" => "5:3",
                "768x1280" => "3:5",
                _ => "16:9", // Default to 16:9
            }
            .to_string()
        });

        CreateTaskRequest {
            model: api_model.to_string(),
            prompt_text: Some(request.prompt.clone()),
            prompt_image: None,
            duration: Some(self.config.default_video_duration),
            ratio,
            seed: None,
            watermark: Some(self.config.watermark),
        }
    }

    /// Transform video response to image generation response
    fn transform_video_to_image_response(
        &self,
        video_response: VideoGenerationResponse,
    ) -> ImageGenerationResponse {
        let data: Vec<ImageData> = video_response
            .video_urls
            .into_iter()
            .map(|url| ImageData {
                url: Some(url),
                b64_json: None,
                revised_prompt: None,
            })
            .collect();

        ImageGenerationResponse {
            created: chrono::Utc::now().timestamp() as u64,
            data,
        }
    }
}
