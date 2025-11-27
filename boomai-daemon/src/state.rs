use boomai_core::ModelProvider;
use std::sync::{Arc, RwLock};

use crate::local::LocalModelManager;

#[derive(Clone)]
pub struct AppState {
    pub model_provider: Arc<RwLock<Arc<dyn ModelProvider>>>,
    pub local_manager: LocalModelManager,
}
