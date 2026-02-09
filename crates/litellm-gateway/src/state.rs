use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct GatewayState {
    pub router: Arc<litellm_router::Router>,
}

impl GatewayState {
    pub fn new(router: litellm_router::Router) -> Self {
        Self {
            router: Arc::new(router),
        }
    }
}
