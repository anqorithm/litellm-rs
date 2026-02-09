use crate::{ExecutionResult, Router, RouterError};

impl Router {
    pub async fn execute(
        &self,
        model_name: &str,
        input: &str,
    ) -> Result<ExecutionResult, RouterError> {
        let deployment = self
            .select_deployment(model_name)
            .ok_or_else(|| RouterError::NoAvailableDeployment(model_name.to_string()))?;

        let output = deployment
            .provider
            .complete(&deployment.model, input)
            .await
            .map_err(|e| RouterError::Provider(e.to_string()))?;

        Ok(ExecutionResult {
            deployment_id: deployment.id.clone(),
            model_used: deployment.model.clone(),
            output,
        })
    }
}
