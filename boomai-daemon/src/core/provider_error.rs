use crate::core::types::ModelId;
use serde::{Deserialize, Serialize};

/// Comprehensive error enum covering all possible provider failure modes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProviderErrorKind {
    Timeout,
    Cancelled,
    AuthMissing,
    AuthInvalid,
    ModelNotFound,
    RateLimited { retry_after_ms: Option<u64> },
    NetworkUnavailable,
    DnsFailure,
    TlsFailure,
    ProxyAuthRequired,
    ServiceUnavailable,
    BadRequest,
    ContextTooLarge { max: usize, got: usize },
    UnsupportedFeature(&'static str),
    Internal(&'static str),
}

/// Structured provider error with visibility tiers
#[derive(Debug)]
pub struct ProviderError {
    pub kind: ProviderErrorKind,
    pub provider_id: ProviderId,
    pub model_id: Option<ModelId>,
    /// User-facing message (sanitized, safe for UI)
    pub user_message: String,
    /// Internal detail (sensitive, logs only)
    pub internal_detail: Option<String>,
    /// Source error (private, never exposed)
    pub source: Option<anyhow::Error>,
}

/// Provider identifier (internal)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProviderId(pub String);

impl ProviderError {
    /// Create a new provider error
    pub fn new(
        kind: ProviderErrorKind,
        provider_id: ProviderId,
        model_id: Option<ModelId>,
        user_message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            provider_id,
            model_id,
            user_message: user_message.into(),
            internal_detail: None,
            source: None,
        }
    }

    /// Add internal details (logs only)
    pub fn with_internal_detail(mut self, detail: impl Into<String>) -> Self {
        self.internal_detail = Some(detail.into());
        self
    }

    /// Add source error (private)
    pub fn with_source(mut self, source: anyhow::Error) -> Self {
        self.source = Some(source);
        self
    }
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref model_id) = self.model_id {
            write!(
                f,
                "[{:?}] {} ({}): {}",
                self.kind,
                self.provider_id.0,
                model_id.as_str(),
                self.user_message
            )
        } else {
            write!(f, "[{:?}] {}: {}", self.kind, self.provider_id.0, self.user_message)
        }
    }
}

impl std::error::Error for ProviderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref())
    }
}
