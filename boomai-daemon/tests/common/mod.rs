use boomai_daemon::core::*;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Test daemon wrapper for integration testing
pub struct TestDaemon {
    registry: Arc<RwLock<ProviderRegistry>>,
    global_limiter: Arc<tokio::sync::Semaphore>,
}

impl TestDaemon {
    pub async fn new() -> Self {
        let registry = Arc::new(RwLock::new(ProviderRegistry::new()));
        let global_limiter = Arc::new(tokio::sync::Semaphore::new(16));
        Self { registry, global_limiter }
    }

    pub async fn with_concurrency_limit(limit: usize) -> Self {
        let registry = Arc::new(RwLock::new(ProviderRegistry::new()));
        let global_limiter = Arc::new(tokio::sync::Semaphore::new(limit));
        Self { registry, global_limiter }
    }

    pub async fn configure_provider(&mut self, provider: Arc<dyn ModelProvider>, model_id: &str) {
        let mut registry = self.registry.write().await;
        registry.register_provider_with_global_limiter(
            ProviderId("test".to_string()),
            provider,
            RunnerConfig::default(),
            ModelId(model_id.to_string()),
            ProviderType::Remote,
            self.global_limiter.clone(),
        );
    }

    pub async fn chat(&self, message: String) -> TestResponse {
        let registry = self.registry.read().await;
        let model_req = ModelRequest {
            messages: vec![crate::core::types::Message {
                role: crate::core::types::Role::User,
                content: message,
            }],
            tools: Vec::new(),
            response_format: None,
            max_output_tokens: None,
            temperature: None,
            top_p: None,
            stop: Vec::new(),
            seed: None,
            stream: false,
            tags: Vec::new(),
            priority: model_request::RequestPriority::Background,
            hard_deadline_ms: None,
            require_json: false,
            truncation: model_request::TruncationPolicy::ErrorIfTooLarge,
        };

        match registry.execute_default(model_req).await {
            Ok(response) => TestResponse::Success(response.content),
            Err(e) => TestResponse::Error(e.user_message),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TestResponse {
    Success(String),
    Error(String),
}

impl TestResponse {
    pub fn is_success(&self) -> bool {
        matches!(self, TestResponse::Success(_))
    }

    pub fn content(&self) -> &str {
        match self {
            TestResponse::Success(content) => content,
            TestResponse::Error(msg) => msg,
        }
    }
}

/// Mock provider for testing different behaviors
pub struct MockProvider {
    pub behavior: MockBehavior,
}

#[derive(Clone)]
pub enum MockBehavior {
    AlwaysSucceeds,
    AlwaysFails(ProviderError),
    DelayedResponse(std::time::Duration),
    Conditional(Box<dyn Fn(&ModelRequest) -> Result<ModelResponse, ProviderError> + Send + Sync>),
}

impl MockProvider {
    pub fn always_succeeds() -> Self {
        Self { behavior: MockBehavior::AlwaysSucceeds }
    }

    pub fn always_fails(error: ProviderError) -> Self {
        Self { behavior: MockBehavior::AlwaysFails(error) }
    }

    pub fn delayed_response(delay: std::time::Duration) -> Self {
        Self { behavior: MockBehavior::DelayedResponse(delay) }
    }

    pub fn conditional<F>(f: F) -> Self
    where
        F: Fn(&ModelRequest) -> Result<ModelResponse, ProviderError> + Send + Sync + 'static,
    {
        Self { behavior: MockBehavior::Conditional(Box::new(f)) }
    }
}

impl ModelProvider for MockProvider {
    async fn chat(&self, req: ModelRequest) -> Result<ModelResponse, ProviderError> {
        match &self.behavior {
            MockBehavior::AlwaysSucceeds => Ok(ModelResponse {
                content: "Mock response".to_string(),
                tool_calls: Vec::new(),
                finish_reason: model_request::FinishReason::Stop,
                usage: model_request::Usage {
                    prompt_tokens: 10,
                    completion_tokens: 20,
                    total_tokens: 30,
                },
                model_id: types::ModelId("mock".to_string()),
                latency_ms: 100,
                warnings: Vec::new(),
            }),
            MockBehavior::AlwaysFails(error) => Err(error.clone()),
            MockBehavior::DelayedResponse(delay) => {
                tokio::time::sleep(*delay).await;
                Ok(ModelResponse {
                    content: "Delayed mock response".to_string(),
                    tool_calls: Vec::new(),
                    finish_reason: model_request::FinishReason::Stop,
                    usage: model_request::Usage {
                        prompt_tokens: 10,
                        completion_tokens: 20,
                        total_tokens: 30,
                    },
                    model_id: types::ModelId("mock".to_string()),
                    latency_ms: delay.as_millis() as u64,
                    warnings: Vec::new(),
                })
            }
            MockBehavior::Conditional(func) => func(&req),
        }
    }
}
