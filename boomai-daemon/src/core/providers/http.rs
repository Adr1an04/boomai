use crate::core::model_request::{ModelRequest, ModelResponse};
use crate::core::provider::ModelProvider;
use crate::core::provider_error::{ProviderError, ProviderErrorKind, ProviderId};
use crate::core::types::ModelId;
use async_trait::async_trait;
use reqwest::Client;

pub struct HttpProvider {
    pub base_url: String,
    pub api_key: Option<String>,
    pub model: String,
    client: Client,
}

impl HttpProvider {
    pub fn new(base_url: String, api_key: Option<String>, model: String) -> Self {
        let client = Client::builder().no_proxy().build().unwrap_or_else(|_| Client::new());

        Self { base_url, api_key, model, client }
    }
}

#[async_trait]
impl ModelProvider for HttpProvider {
    async fn chat(&self, req: ModelRequest) -> Result<ModelResponse, ProviderError> {
        let start = std::time::Instant::now();
        let mut body = serde_json::json!({
            "model": self.model,
            "messages": req.messages
        });

        // optional parameters
        if let Some(max_tokens) = req.max_output_tokens {
            body["max_tokens"] = serde_json::json!(max_tokens);
        }
        if let Some(temp) = req.temperature {
            body["temperature"] = serde_json::json!(temp);
        }
        if let Some(top_p) = req.top_p {
            body["top_p"] = serde_json::json!(top_p);
        }
        if !req.stop.is_empty() {
            body["stop"] = serde_json::json!(req.stop);
        }
        if let Some(seed) = req.seed {
            body["seed"] = serde_json::json!(seed);
        }
        if req.stream {
            body["stream"] = serde_json::json!(true);
        }

        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let mut request = self.client.post(&url).json(&body);

        if let Some(key) = &self.api_key {
            if !key.is_empty() {
                request = request.header("Authorization", format!("Bearer {}", key));
            }
        }

        let response = match request.send().await {
            Ok(resp) => resp,
            Err(e) => {
                return Err(ProviderError::new(
                    ProviderErrorKind::NetworkUnavailable,
                    ProviderId(self.base_url.clone()),
                    Some(ModelId(self.model.clone())),
                    "Failed to send request to provider",
                )
                .with_source(anyhow::Error::from(e)));
            }
        };

        let duration_ms = start.elapsed().as_millis();

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            let error_kind = match status.as_u16() {
                401 => ProviderErrorKind::AuthInvalid,
                403 => ProviderErrorKind::AuthInvalid,
                404 => ProviderErrorKind::ModelNotFound,
                429 => ProviderErrorKind::RateLimited { retry_after_ms: None },
                500..=599 => ProviderErrorKind::ServiceUnavailable,
                _ => ProviderErrorKind::BadRequest,
            };

            return Err(ProviderError::new(
                error_kind,
                ProviderId(self.base_url.clone()),
                Some(ModelId(self.model.clone())),
                "API request failed",
            )
            .with_internal_detail(format!("HTTP {}: {}", status, error_text)));
        }

        let raw_text = match response.text().await {
            Ok(text) => text,
            Err(e) => {
                return Err(ProviderError::new(
                    ProviderErrorKind::NetworkUnavailable,
                    ProviderId(self.base_url.clone()),
                    Some(ModelId(self.model.clone())),
                    "Failed to read response from provider",
                )
                .with_source(anyhow::Error::from(e)));
            }
        };

        let data: serde_json::Value = match serde_json::from_str(&raw_text) {
            Ok(data) => data,
            Err(e) => {
                return Err(ProviderError::new(
                    ProviderErrorKind::BadRequest,
                    ProviderId(self.base_url.clone()),
                    Some(ModelId(self.model.clone())),
                    "Invalid response format from provider",
                )
                .with_source(anyhow::Error::from(e)));
            }
        };

        let content = data["choices"][0]["message"]["content"].as_str().ok_or_else(|| {
            ProviderError::new(
                ProviderErrorKind::BadRequest,
                ProviderId(self.base_url.clone()),
                Some(ModelId(self.model.clone())),
                "Invalid response format: missing content",
            )
        })?;

        let tool_calls = Vec::new(); // TODO: Parse tool calls from response
        let finish_reason = data["choices"][0]["finish_reason"].as_str().unwrap_or("stop");

        let finish_reason = match finish_reason {
            "stop" => crate::core::model_request::FinishReason::Stop,
            "length" => crate::core::model_request::FinishReason::Length,
            "tool_calls" => crate::core::model_request::FinishReason::ToolCalls,
            "content_filter" => crate::core::model_request::FinishReason::ContentFilter,
            _ => crate::core::model_request::FinishReason::Stop,
        };

        tracing::debug!(
            target: "llm_trace",
            model = %self.model,
            duration_ms = duration_ms,
            prompt_messages = req.messages.len(),
            raw_response = %raw_text,
            "llm_call"
        );

        Ok(ModelResponse {
            content: content.to_string(),
            tool_calls,
            finish_reason,
            usage: crate::core::model_request::Usage {
                prompt_tokens: 0,     // TODO: Parse from response
                completion_tokens: 0, // TODO: Parse from response
                total_tokens: 0,      // TODO: Parse from response
            },
            model_id: ModelId(self.model.clone()),
            latency_ms: duration_ms as u64,
            warnings: Vec::new(),
        })
    }
}
