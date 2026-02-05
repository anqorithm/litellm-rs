//! Fine-tuning Manager
//!
//! Manages fine-tuning providers and provides a unified interface.

use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{debug, info};

use super::config::FineTuningConfig;
use super::providers::{BoxedFineTuningProvider, FineTuningError, FineTuningResult};
use super::types::{
    CreateJobRequest, FineTuningCheckpoint, FineTuningJob, ListEventsParams, ListEventsResponse,
    ListJobsParams, ListJobsResponse,
};

/// Fine-tuning manager
///
/// Manages multiple fine-tuning providers and provides a unified interface
/// for creating and managing fine-tuning jobs.
pub struct FineTuningManager {
    /// Configuration
    config: FineTuningConfig,
    /// Registered providers
    providers: RwLock<HashMap<String, BoxedFineTuningProvider>>,
}

impl FineTuningManager {
    /// Create a new fine-tuning manager
    pub fn new(config: FineTuningConfig) -> Self {
        Self {
            config,
            providers: RwLock::new(HashMap::new()),
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(FineTuningConfig::default())
    }

    /// Register a fine-tuning provider
    pub async fn register_provider(
        &self,
        name: impl Into<String>,
        provider: BoxedFineTuningProvider,
    ) {
        let name = name.into();
        info!("Registering fine-tuning provider: {}", name);
        self.providers.write().await.insert(name, provider);
    }

    /// Unregister a provider
    pub async fn unregister_provider(&self, name: &str) -> bool {
        let removed = self.providers.write().await.remove(name).is_some();
        if removed {
            debug!("Unregistered fine-tuning provider: {}", name);
        }
        removed
    }

    /// List registered providers
    pub async fn list_providers(&self) -> Vec<String> {
        self.providers.read().await.keys().cloned().collect()
    }

    /// Check if a provider is registered
    pub async fn has_provider(&self, name: &str) -> bool {
        self.providers.read().await.contains_key(name)
    }

    /// Get the default provider name
    pub fn default_provider(&self) -> Option<&str> {
        self.config.default_provider.as_deref()
    }

    /// Get a provider by name
    async fn get_provider(&self, name: &str) -> FineTuningResult<BoxedFineTuningProvider> {
        self.providers
            .read()
            .await
            .get(name)
            .cloned()
            .ok_or_else(|| FineTuningError::provider_not_found(name))
    }

    /// Resolve provider name (use default if not specified)
    fn resolve_provider_name<'a>(&'a self, provider: Option<&'a str>) -> FineTuningResult<&'a str> {
        provider
            .or(self.config.default_provider.as_deref())
            .ok_or_else(|| {
                FineTuningError::invalid_request("No provider specified and no default configured")
            })
    }

    /// Create a fine-tuning job
    pub async fn create_job(
        &self,
        provider_name: Option<&str>,
        request: CreateJobRequest,
    ) -> FineTuningResult<FineTuningJob> {
        let provider_name = self.resolve_provider_name(provider_name)?;
        let provider = self.get_provider(provider_name).await?;

        debug!(
            "Creating fine-tuning job with provider {} for model {}",
            provider_name, request.model
        );

        provider.create_job(request).await
    }

    /// List fine-tuning jobs
    pub async fn list_jobs(
        &self,
        provider_name: Option<&str>,
        params: ListJobsParams,
    ) -> FineTuningResult<ListJobsResponse> {
        let provider_name = self.resolve_provider_name(provider_name)?;
        let provider = self.get_provider(provider_name).await?;

        provider.list_jobs(params).await
    }

    /// Get a specific fine-tuning job
    pub async fn get_job(
        &self,
        provider_name: Option<&str>,
        job_id: &str,
    ) -> FineTuningResult<FineTuningJob> {
        let provider_name = self.resolve_provider_name(provider_name)?;
        let provider = self.get_provider(provider_name).await?;

        provider.get_job(job_id).await
    }

    /// Cancel a fine-tuning job
    pub async fn cancel_job(
        &self,
        provider_name: Option<&str>,
        job_id: &str,
    ) -> FineTuningResult<FineTuningJob> {
        let provider_name = self.resolve_provider_name(provider_name)?;
        let provider = self.get_provider(provider_name).await?;

        info!(
            "Cancelling fine-tuning job {} on provider {}",
            job_id, provider_name
        );

        provider.cancel_job(job_id).await
    }

    /// List events for a fine-tuning job
    pub async fn list_events(
        &self,
        provider_name: Option<&str>,
        job_id: &str,
        params: ListEventsParams,
    ) -> FineTuningResult<ListEventsResponse> {
        let provider_name = self.resolve_provider_name(provider_name)?;
        let provider = self.get_provider(provider_name).await?;

        provider.list_events(job_id, params).await
    }

    /// List checkpoints for a fine-tuning job
    pub async fn list_checkpoints(
        &self,
        provider_name: Option<&str>,
        job_id: &str,
    ) -> FineTuningResult<Vec<FineTuningCheckpoint>> {
        let provider_name = self.resolve_provider_name(provider_name)?;
        let provider = self.get_provider(provider_name).await?;

        provider.list_checkpoints(job_id).await
    }

    /// List jobs from all providers
    pub async fn list_all_jobs(
        &self,
        params: ListJobsParams,
    ) -> HashMap<String, FineTuningResult<ListJobsResponse>> {
        let providers = self.providers.read().await;
        let mut results = HashMap::new();

        for (name, provider) in providers.iter() {
            let result = provider.list_jobs(params.clone()).await;
            results.insert(name.clone(), result);
        }

        results
    }

    /// Wait for a job to complete
    pub async fn wait_for_job(
        &self,
        provider_name: Option<&str>,
        job_id: &str,
        poll_interval_seconds: Option<u64>,
    ) -> FineTuningResult<FineTuningJob> {
        let provider_name = self.resolve_provider_name(provider_name)?;
        let interval = poll_interval_seconds.unwrap_or(self.config.poll_interval_seconds);

        loop {
            let job = self.get_job(Some(provider_name), job_id).await?;

            if job.is_terminal() {
                return Ok(job);
            }

            debug!(
                "Job {} status: {}, waiting {}s before next poll",
                job_id, job.status, interval
            );

            tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
        }
    }
}

impl Default for FineTuningManager {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::fine_tuning::providers::FineTuningProvider;
    use crate::core::fine_tuning::types::{FineTuningJobStatus, ListJobsResponse};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// Mock provider for testing
    struct MockFineTuningProvider {
        name: &'static str,
        create_count: AtomicU32,
        list_count: AtomicU32,
    }

    impl MockFineTuningProvider {
        fn new(name: &'static str) -> Self {
            Self {
                name,
                create_count: AtomicU32::new(0),
                list_count: AtomicU32::new(0),
            }
        }
    }

    #[async_trait::async_trait]
    impl FineTuningProvider for MockFineTuningProvider {
        fn name(&self) -> &'static str {
            self.name
        }

        async fn create_job(&self, request: CreateJobRequest) -> FineTuningResult<FineTuningJob> {
            self.create_count.fetch_add(1, Ordering::SeqCst);

            Ok(FineTuningJob {
                id: format!("ftjob-{}", self.create_count.load(Ordering::SeqCst)),
                object: "fine_tuning.job".to_string(),
                model: request.model,
                fine_tuned_model: None,
                organization_id: None,
                status: FineTuningJobStatus::Queued,
                training_file: request.training_file,
                validation_file: request.validation_file,
                result_files: vec![],
                hyperparameters: request.hyperparameters,
                trained_tokens: None,
                error: None,
                created_at: chrono::Utc::now().timestamp(),
                started_at: None,
                finished_at: None,
                estimated_finish: None,
                suffix: request.suffix,
                seed: request.seed,
                metadata: request.metadata,
                provider: Some(self.name.to_string()),
            })
        }

        async fn list_jobs(&self, _params: ListJobsParams) -> FineTuningResult<ListJobsResponse> {
            self.list_count.fetch_add(1, Ordering::SeqCst);

            Ok(ListJobsResponse {
                object: "list".to_string(),
                data: vec![],
                has_more: false,
            })
        }

        async fn get_job(&self, job_id: &str) -> FineTuningResult<FineTuningJob> {
            Ok(FineTuningJob {
                id: job_id.to_string(),
                object: "fine_tuning.job".to_string(),
                model: "gpt-3.5-turbo".to_string(),
                fine_tuned_model: None,
                organization_id: None,
                status: FineTuningJobStatus::Succeeded,
                training_file: "file-abc".to_string(),
                validation_file: None,
                result_files: vec![],
                hyperparameters: None,
                trained_tokens: None,
                error: None,
                created_at: 0,
                started_at: None,
                finished_at: None,
                estimated_finish: None,
                suffix: None,
                seed: None,
                metadata: HashMap::new(),
                provider: Some(self.name.to_string()),
            })
        }

        async fn cancel_job(&self, job_id: &str) -> FineTuningResult<FineTuningJob> {
            let mut job = self.get_job(job_id).await?;
            job.status = FineTuningJobStatus::Cancelled;
            Ok(job)
        }

        async fn list_events(
            &self,
            _job_id: &str,
            _params: ListEventsParams,
        ) -> FineTuningResult<ListEventsResponse> {
            Ok(ListEventsResponse {
                object: "list".to_string(),
                data: vec![],
                has_more: false,
            })
        }
    }

    #[tokio::test]
    async fn test_manager_creation() {
        let manager = FineTuningManager::with_defaults();
        assert!(manager.list_providers().await.is_empty());
    }

    #[tokio::test]
    async fn test_register_provider() {
        let manager = FineTuningManager::with_defaults();
        let provider = Arc::new(MockFineTuningProvider::new("mock"));

        manager.register_provider("mock", provider).await;

        assert!(manager.has_provider("mock").await);
        assert_eq!(manager.list_providers().await, vec!["mock"]);
    }

    #[tokio::test]
    async fn test_unregister_provider() {
        let manager = FineTuningManager::with_defaults();
        let provider = Arc::new(MockFineTuningProvider::new("mock"));

        manager.register_provider("mock", provider).await;
        assert!(manager.has_provider("mock").await);

        let removed = manager.unregister_provider("mock").await;
        assert!(removed);
        assert!(!manager.has_provider("mock").await);
    }

    #[tokio::test]
    async fn test_create_job() {
        let manager = FineTuningManager::with_defaults();
        let provider = Arc::new(MockFineTuningProvider::new("mock"));

        manager.register_provider("mock", provider.clone()).await;

        let request = CreateJobRequest::new("gpt-3.5-turbo", "file-abc123");
        let job = manager.create_job(Some("mock"), request).await.unwrap();

        assert_eq!(job.model, "gpt-3.5-turbo");
        assert_eq!(job.training_file, "file-abc123");
        assert_eq!(job.provider, Some("mock".to_string()));
        assert_eq!(provider.create_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_list_jobs() {
        let manager = FineTuningManager::with_defaults();
        let provider = Arc::new(MockFineTuningProvider::new("mock"));

        manager.register_provider("mock", provider.clone()).await;

        let response = manager
            .list_jobs(Some("mock"), ListJobsParams::new())
            .await
            .unwrap();

        assert_eq!(response.object, "list");
        assert_eq!(provider.list_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_get_job() {
        let manager = FineTuningManager::with_defaults();
        let provider = Arc::new(MockFineTuningProvider::new("mock"));

        manager.register_provider("mock", provider).await;

        let job = manager.get_job(Some("mock"), "ftjob-123").await.unwrap();

        assert_eq!(job.id, "ftjob-123");
    }

    #[tokio::test]
    async fn test_cancel_job() {
        let manager = FineTuningManager::with_defaults();
        let provider = Arc::new(MockFineTuningProvider::new("mock"));

        manager.register_provider("mock", provider).await;

        let job = manager.cancel_job(Some("mock"), "ftjob-123").await.unwrap();

        assert_eq!(job.status, FineTuningJobStatus::Cancelled);
    }

    #[tokio::test]
    async fn test_default_provider() {
        let config = FineTuningConfig::new().default_provider("mock");
        let manager = FineTuningManager::new(config);
        let provider = Arc::new(MockFineTuningProvider::new("mock"));

        manager.register_provider("mock", provider).await;

        // Should use default provider when None is passed
        let request = CreateJobRequest::new("gpt-3.5-turbo", "file-abc123");
        let job = manager.create_job(None, request).await.unwrap();

        assert_eq!(job.provider, Some("mock".to_string()));
    }

    #[tokio::test]
    async fn test_provider_not_found() {
        let manager = FineTuningManager::with_defaults();

        let request = CreateJobRequest::new("gpt-3.5-turbo", "file-abc123");
        let result = manager.create_job(Some("nonexistent"), request).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            FineTuningError::ProviderNotFound { .. }
        ));
    }

    #[tokio::test]
    async fn test_no_default_provider() {
        let manager = FineTuningManager::with_defaults();

        let request = CreateJobRequest::new("gpt-3.5-turbo", "file-abc123");
        let result = manager.create_job(None, request).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_all_jobs() {
        let manager = FineTuningManager::with_defaults();

        manager
            .register_provider(
                "provider1",
                Arc::new(MockFineTuningProvider::new("provider1")),
            )
            .await;
        manager
            .register_provider(
                "provider2",
                Arc::new(MockFineTuningProvider::new("provider2")),
            )
            .await;

        let results = manager.list_all_jobs(ListJobsParams::new()).await;

        assert_eq!(results.len(), 2);
        assert!(results.contains_key("provider1"));
        assert!(results.contains_key("provider2"));
    }
}
