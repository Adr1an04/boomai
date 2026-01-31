use crate::core::model_request::{ModelRequest, ModelResponse};
use crate::core::provider::ModelProvider;
use crate::core::provider_error::{ProviderError, ProviderId};
use crate::core::provider_runner::{ProviderRunner, RunnerConfig};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct ProviderRegistry {
    providers: HashMap<ProviderId, ProviderEntry>,
    default_provider: Option<ProviderId>,
}

#[derive(Clone)]
pub struct ProviderEntry {
    pub runner: Arc<ProviderRunner>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self { providers: HashMap::new(), default_provider: None }
    }

    pub fn register_provider(
        &mut self,
        provider_id: ProviderId,
        provider: Arc<dyn ModelProvider>,
        runner_config: RunnerConfig,
    ) {
        let runner = Arc::new(ProviderRunner::new(provider, runner_config));

        let entry = ProviderEntry { runner };

        self.providers.insert(provider_id.clone(), entry);

        if self.default_provider.is_none() {
            self.default_provider = Some(provider_id);
        }
    }

    pub fn register_provider_with_global_limiter(
        &mut self,
        provider_id: ProviderId,
        provider: Arc<dyn ModelProvider>,
        runner_config: RunnerConfig,
        global_limiter: Arc<tokio::sync::Semaphore>,
    ) {
        let runner = Arc::new(
            ProviderRunner::new(provider, runner_config).with_global_limiter(global_limiter),
        );

        let entry = ProviderEntry { runner };

        self.providers.insert(provider_id.clone(), entry);

        if self.default_provider.is_none() {
            self.default_provider = Some(provider_id);
        }
    }

    pub fn set_default(&mut self, provider_id: ProviderId) {
        if self.providers.contains_key(&provider_id) {
            self.default_provider = Some(provider_id);
        }
    }

    pub fn get_default_runner(&self) -> Option<Arc<ProviderRunner>> {
        self.default_provider
            .as_ref()
            .and_then(|id| self.providers.get(id))
            .map(|entry| entry.runner.clone())
    }

    pub async fn execute_default(
        &self,
        request: ModelRequest,
    ) -> Result<ModelResponse, ProviderError> {
        let runner = self.get_default_runner().ok_or_else(|| {
            ProviderError::new(
                crate::core::provider_error::ProviderErrorKind::Internal("no_default_provider"),
                ProviderId("registry".to_string()),
                None,
                "No default provider configured",
            )
        })?;

        runner.execute(request).await
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
