mod deployment;
mod execute_impl;
mod gateway_config;

pub use deployment::Deployment;

#[derive(Debug, thiserror::Error)]
pub enum RouterError {
    #[error("no available deployment for model: {0}")]
    NoAvailableDeployment(String),
    #[error("provider error: {0}")]
    Provider(String),
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub deployment_id: String,
    pub model_used: String,
    pub output: String,
}

#[derive(Debug, Default, Clone)]
pub struct Router {
    deployments: Vec<Deployment>,
}

impl Router {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_deployment(&mut self, deployment: Deployment) {
        self.deployments.push(deployment);
    }

    pub fn select_deployment(&self, model_name: &str) -> Option<&Deployment> {
        self.deployments
            .iter()
            .find(|deployment| {
                deployment.model_name == model_name || deployment.model == model_name
            })
            .or_else(|| self.deployments.first())
    }

    pub fn len(&self) -> usize {
        self.deployments.len()
    }

    pub fn is_empty(&self) -> bool {
        self.deployments.is_empty()
    }
}
