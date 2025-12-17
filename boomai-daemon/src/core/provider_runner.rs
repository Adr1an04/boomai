use crate::core::model_request::{ModelRequest, ModelResponse};
use crate::core::provider::ModelProvider;
use crate::core::provider_error::{ProviderError, ProviderErrorKind, ProviderId};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::{timeout, Duration};
use tokio_util::sync::CancellationToken;

/// provider runner uses timeouts, retries, and concurrency limits to manage requests
#[derive(Clone)]
pub struct ProviderRunner {
    provider: Arc<dyn ModelProvider>,
    config: RunnerConfig,
    concurrency_limiter: Arc<Semaphore>,
    global_limiter: Option<Arc<Semaphore>>, // global concurrency limiter
}

/// config for provider runner
#[derive(Debug, Clone)]
pub struct RunnerConfig {
    pub request_timeout_ms: u64,
    pub max_concurrent: usize,
    pub cancellation_token: Option<CancellationToken>,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self { request_timeout_ms: 60_000, max_concurrent: 8, cancellation_token: None }
    }
}

impl ProviderRunner {
    /// Create a new ProviderRunner with the given provider and config
    pub fn new(provider: Arc<dyn ModelProvider>, config: RunnerConfig) -> Self {
        let concurrency_limiter = Arc::new(Semaphore::new(config.max_concurrent));
        Self { provider, config, concurrency_limiter, global_limiter: None }
    }

    /// Execute a request with policy enforcement (timeouts, retries, concurrency control)
    pub async fn execute(&self, request: ModelRequest) -> Result<ModelResponse, ProviderError> {
        // Apply concurrency limiting
        let _permit = self.concurrency_limiter.acquire().await.map_err(|_| {
            ProviderError::new(
                ProviderErrorKind::Internal("concurrency_limiter"),
                ProviderId("unknown".to_string()),
                None,
                "Failed to acquire concurrency permit",
            )
        })?;

        // Apply global concurrency limiting if configured
        let _global_permit = if let Some(ref global_limiter) = self.global_limiter {
            Some(global_limiter.acquire().await.map_err(|_| {
                ProviderError::new(
                    ProviderErrorKind::Internal("global_limiter"),
                    ProviderId("unknown".to_string()),
                    None,
                    "Failed to acquire global concurrency permit",
                )
            })?)
        } else {
            None
        };

        // Check cancellation token
        if let Some(ref token) = self.config.cancellation_token {
            if token.is_cancelled() {
                return Err(ProviderError::new(
                    ProviderErrorKind::Cancelled,
                    ProviderId("unknown".to_string()),
                    None,
                    "Request cancelled",
                ));
            }
        }

        // Execute with timeout
        let timeout_duration = Duration::from_millis(self.config.request_timeout_ms);
        timeout(timeout_duration, self.provider.chat(request)).await.map_err(|_| {
            ProviderError::new(
                ProviderErrorKind::Timeout,
                ProviderId("unknown".to_string()),
                None,
                "Request timed out",
            )
        })?
    }

    /// Set global concurrency limiter
    pub fn with_global_limiter(mut self, global_limiter: Arc<Semaphore>) -> Self {
        self.global_limiter = Some(global_limiter);
        self
    }
}
