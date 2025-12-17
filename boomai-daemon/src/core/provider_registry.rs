use crate::core::model_request::{ModelRequest, ModelResponse};
use crate::core::provider::ModelProvider;
use crate::core::provider_error::{ProviderError, ProviderId};
use crate::core::provider_runner::{ProviderRunner, RunnerConfig};
use crate::core::types::ModelId;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct ProviderRegistry {
    providers: HashMap<ProviderId, ProviderEntry>,
    default_provider: Option<ProviderId>,
}

#[derive(Clone)]
pub struct ProviderEntry {
    pub provider: Arc<dyn ModelProvider>,
    pub runner: Arc<ProviderRunner>,
    pub model_id: ModelId,
    pub provider_type: ProviderType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProviderType {
    Local,
    Remote,
    Mock,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            default_provider: None,
        }
    }

    pub fn with_global_limiter(_global_limiter: Arc<tokio::sync::Semaphore>) -> Self {
        Self {
            providers: HashMap::new(),
            default_provider: None,
        }
    }

    pub fn register_provider(
        &mut self,
        provider_id: ProviderId,
        provider: Arc<dyn ModelProvider>,
        runner_config: RunnerConfig,
        model_id: ModelId,
        provider_type: ProviderType,
    ) {
        let runner = Arc::new(ProviderRunner::new(provider.clone(), runner_config));

        let entry = ProviderEntry {
            provider,
            runner,
            model_id,
            provider_type,
        };

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
        model_id: ModelId,
        provider_type: ProviderType,
        global_limiter: Arc<tokio::sync::Semaphore>,
    ) {
        let runner = Arc::new(
            ProviderRunner::new(provider.clone(), runner_config)
                .with_global_limiter(global_limiter)
        );

        let entry = ProviderEntry {
            provider,
            runner,
            model_id,
            provider_type,
        };

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

    pub fn get_runner(&self, provider_id: &ProviderId) -> Option<Arc<ProviderRunner>> {
        self.providers.get(provider_id).map(|entry| entry.runner.clone())
    }

    pub async fn execute_default(&self, request: ModelRequest) -> Result<ModelResponse, ProviderError> {
        let runner = self.get_default_runner()
            .ok_or_else(|| ProviderError::new(
                crate::core::provider_error::ProviderErrorKind::Internal("no_default_provider"),
                ProviderId("registry".to_string()),
                None,
                "No default provider configured",
            ))?;

        runner.execute(request).await
    }

    pub fn list_providers(&self) -> Vec<(ProviderId, ModelId, ProviderType)> {
        self.providers.iter()
            .map(|(id, entry)| (id.clone(), entry.model_id.clone(), entry.provider_type.clone()))
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }

    pub fn total_concurrency(&self) -> usize {
        self.providers.values()
            .map(|entry| entry.runner.current_concurrency())
            .sum()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
