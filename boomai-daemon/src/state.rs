use boomai_core::ModelProvider;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub model_provider: Arc<dyn ModelProvider>,
}

