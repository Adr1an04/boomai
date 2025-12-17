use crate::core::model_request::{ModelRequest, ModelResponse};
use crate::core::provider_error::ProviderError;
use async_trait::async_trait;

#[async_trait]
pub trait ModelProvider: Send + Sync {
    async fn chat(&self, req: ModelRequest) -> Result<ModelResponse, ProviderError>;
}
