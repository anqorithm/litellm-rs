use litellm_provider_core::DynProvider;

#[derive(Clone)]
pub struct Deployment {
    pub id: String,
    pub model: String,
    pub model_name: String,
    pub provider: DynProvider,
}

impl std::fmt::Debug for Deployment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Deployment")
            .field("id", &self.id)
            .field("model", &self.model)
            .field("model_name", &self.model_name)
            .field("provider_id", &self.provider.id())
            .finish()
    }
}

impl Deployment {
    pub fn new(id: impl Into<String>, model: impl Into<String>, provider: DynProvider) -> Self {
        let model = model.into();
        Self {
            id: id.into(),
            model_name: model.clone(),
            model,
            provider,
        }
    }
}
