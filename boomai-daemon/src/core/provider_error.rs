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
    pub user_message: String,                    // sanitized, safe for UI
    pub internal_detail: Option<String>,        // sensitive, logs only
    pub source: Option<anyhow::Error>,          // private, never exposed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizedProviderError {
    pub kind: SanitizedErrorKind,
    pub provider_id: SanitizedProviderId,
    pub model_id: Option<SanitizedModelId>,
    pub user_message: String,
}

/// Sanitized error kind for UI exposure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SanitizedErrorKind {
    Timeout,
    Authentication,
    Authorization,
    Configuration,
    Network,
    Service,
    Request,
    Unsupported,
    Internal,
}

/// Sanitized identifiers (no sensitive URLs/paths)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizedProviderId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizedModelId(pub String);

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

    /// Convert to sanitized version for API responses
    pub fn sanitized(&self) -> SanitizedProviderError {
        SanitizedProviderError {
            kind: self.kind.sanitized(),
            provider_id: SanitizedProviderId(self.provider_id.0.clone()),
            model_id: self.model_id.as_ref().map(|id| SanitizedModelId(id.as_str().to_string())),
            user_message: self.user_message.clone(),
        }
    }
}

impl ProviderErrorKind {
    /// Convert to sanitized error kind
    pub fn sanitized(&self) -> SanitizedErrorKind {
        match self {
            ProviderErrorKind::Timeout => SanitizedErrorKind::Timeout,
            ProviderErrorKind::Cancelled => SanitizedErrorKind::Request,
            ProviderErrorKind::AuthMissing | ProviderErrorKind::AuthInvalid => {
                SanitizedErrorKind::Authentication
            }
            ProviderErrorKind::ModelNotFound => SanitizedErrorKind::Configuration,
            ProviderErrorKind::RateLimited { .. } => SanitizedErrorKind::Service,
            ProviderErrorKind::NetworkUnavailable
            | ProviderErrorKind::DnsFailure
            | ProviderErrorKind::TlsFailure
            | ProviderErrorKind::ProxyAuthRequired => SanitizedErrorKind::Network,
            ProviderErrorKind::ServiceUnavailable => SanitizedErrorKind::Service,
            ProviderErrorKind::BadRequest => SanitizedErrorKind::Request,
            ProviderErrorKind::ContextTooLarge { .. } => SanitizedErrorKind::Request,
            ProviderErrorKind::UnsupportedFeature(_) => SanitizedErrorKind::Unsupported,
            ProviderErrorKind::Internal(_) => SanitizedErrorKind::Internal,
        }
    }
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.provider_id.0, self.user_message)
    }
}

impl std::error::Error for ProviderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::StatusCode;

    #[test]
    fn test_error_mapping_http_codes() {
        // Test authentication errors
        assert_eq!(ProviderErrorKind::AuthInvalid.sanitized(), SanitizedErrorKind::Authentication);
        assert_eq!(ProviderErrorKind::AuthMissing.sanitized(), SanitizedErrorKind::Authentication);

        // Test network errors
        assert_eq!(ProviderErrorKind::NetworkUnavailable.sanitized(), SanitizedErrorKind::Network);
        assert_eq!(ProviderErrorKind::DnsFailure.sanitized(), SanitizedErrorKind::Network);
        assert_eq!(ProviderErrorKind::TlsFailure.sanitized(), SanitizedErrorKind::Network);

        // Test service errors
        assert_eq!(ProviderErrorKind::ServiceUnavailable.sanitized(), SanitizedErrorKind::Service);
        assert_eq!(ProviderErrorKind::RateLimited { retry_after_ms: None }.sanitized(), SanitizedErrorKind::Service);

        // Test configuration errors
        assert_eq!(ProviderErrorKind::ModelNotFound.sanitized(), SanitizedErrorKind::Configuration);

        // Test timeout
        assert_eq!(ProviderErrorKind::Timeout.sanitized(), SanitizedErrorKind::Timeout);
    }

    #[test]
    fn test_error_sanitization() {
        let error = ProviderError::new(
            ProviderErrorKind::AuthInvalid,
            ProviderId("openai".to_string()),
            Some(ModelId("gpt-4".to_string())),
            "Authentication failed",
        ).with_internal_detail("API key sk-1234567890abcdef is invalid");

        let sanitized = error.sanitized();

        // Should not contain sensitive details
        assert!(!sanitized.user_message.contains("sk-1234567890abcdef"));
        assert_eq!(sanitized.kind, SanitizedErrorKind::Authentication);
        assert_eq!(sanitized.provider_id.0, "openai");
        assert_eq!(sanitized.model_id.as_ref().expect("model_id should be present").0, "gpt-4");
    }

    #[test]
    fn test_error_creation() {
        let error = ProviderError::new(
            ProviderErrorKind::Timeout,
            ProviderId("test".to_string()),
            None,
            "Request timed out",
        );

        assert_eq!(error.kind, ProviderErrorKind::Timeout);
        assert_eq!(error.provider_id.0, "test");
        assert_eq!(error.user_message, "Request timed out");
        assert!(error.model_id.is_none());
    }

    #[test]
    fn test_error_with_internal_detail() {
        let error = ProviderError::new(
            ProviderErrorKind::NetworkUnavailable,
            ProviderId("test".to_string()),
            None,
            "Network error",
        ).with_internal_detail("Connection refused on port 8080");

        assert_eq!(error.internal_detail, Some("Connection refused on port 8080".to_string()));
        // Internal details should not appear in sanitized version
        let sanitized = error.sanitized();
        assert!(!sanitized.user_message.contains("port 8080"));
    }

    #[test]
    fn test_context_too_large_error() {
        let error = ProviderErrorKind::ContextTooLarge { max: 4096, got: 8192 };
        assert_eq!(error.sanitized(), SanitizedErrorKind::Request);

        match error {
            ProviderErrorKind::ContextTooLarge { max, got } => {
                assert_eq!(max, 4096);
                assert_eq!(got, 8192);
            }
            other => panic!("Expected ContextTooLarge, got {:?}", other),
        }
    }

    #[test]
    fn test_rate_limited_error() {
        let error = ProviderErrorKind::RateLimited { retry_after_ms: Some(5000) };
        assert_eq!(error.sanitized(), SanitizedErrorKind::Service);

        match error {
            ProviderErrorKind::RateLimited { retry_after_ms } => {
                assert_eq!(retry_after_ms, Some(5000));
            }
            other => panic!("Expected RateLimited, got {:?}", other),
        }
    }
}