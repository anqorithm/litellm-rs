#[derive(Debug, Clone)]
pub struct ApiState {
    pub router: litellm_router::Router,
}

impl ApiState {
    pub fn new(router: litellm_router::Router) -> Self {
        Self { router }
    }
}
