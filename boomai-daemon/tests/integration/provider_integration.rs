use boomai_daemon::tests::common::{MockProvider, TestDaemon};
use boomai_daemon::core::provider_error::{ProviderError, ProviderErrorKind, ProviderId};
use boomai_daemon::core::ProviderRegistry;
use std::time::Duration;

#[tokio::test]
async fn test_registry_fallback_on_failure() {
    let mut daemon = TestDaemon::new().await;

    // Configure primary provider that fails
    let failing_provider = MockProvider::always_fails(ProviderError::new(
        ProviderErrorKind::ServiceUnavailable,
        ProviderId("primary".to_string()),
        None,
        "Primary provider down",
    ));
    daemon.configure_provider(failing_provider.into(), "fail").await;

    // Configure backup provider that succeeds
    let working_provider = MockProvider::always_succeeds();
    daemon.configure_provider(working_provider.into(), "work").await;

    let response = daemon.chat("Test fallback".into()).await;
    assert!(response.is_success());
    assert_eq!(response.content(), "Mock response");
}

#[tokio::test]
async fn test_global_concurrency_backpressure() {
    let mut daemon = TestDaemon::with_concurrency_limit(2).await;

    // Configure a provider with delay to test queuing
    let delayed_provider = MockProvider::delayed_response(Duration::from_millis(100));
    daemon.configure_provider(delayed_provider.into(), "delayed").await;

    // Launch 5 concurrent requests
    let futures: Vec<_> = (0..5).map(|i| {
        let d = daemon.clone();
        tokio::spawn(async move {
            d.chat(format!("Request {}", i)).await
        })
    }).collect();

    let start = std::time::Instant::now();
    let results = futures::future::join_all(futures).await;
    let elapsed = start.elapsed();

    // Should take longer due to queuing (2 at a time, 100ms each)
    // 3 batches: 300ms minimum
    assert!(elapsed > Duration::from_millis(250));

    // All should succeed eventually
    for result in results {
        let response = result.unwrap();
        assert!(response.is_success());
    }
}

#[tokio::test]
async fn test_provider_error_handling() {
    let mut daemon = TestDaemon::new().await;

    // Configure provider that always fails with auth error
    let auth_error = ProviderError::new(
        ProviderErrorKind::AuthInvalid,
        ProviderId("test".to_string()),
        None,
        "Invalid API key",
    );
    let failing_provider = MockProvider::always_fails(auth_error);
    daemon.configure_provider(failing_provider.into(), "auth_fail").await;

    let response = daemon.chat("Test auth failure".into()).await;
    assert!(!response.is_success());
    assert!(response.content().contains("Invalid API key"));
}

#[tokio::test]
async fn test_provider_timeout_handling() {
    let mut daemon = TestDaemon::new().await;

    // Configure provider with long delay
    let timeout_provider = MockProvider::delayed_response(Duration::from_secs(10));
    daemon.configure_provider(timeout_provider.into(), "timeout").await;

    let start = std::time::Instant::now();
    let response = daemon.chat("Test timeout".into()).await;
    let elapsed = start.elapsed();

    // Should timeout within reasonable time (not 10 seconds)
    assert!(elapsed < Duration::from_secs(5));
    assert!(!response.is_success());
}

#[tokio::test]
async fn test_concurrent_requests_isolation() {
    let mut daemon = TestDaemon::new().await;

    // Configure two different providers
    let fast_provider = MockProvider::conditional(|req| {
        if req.messages[0].content.contains("fast") {
            Ok(boomai_daemon::core::model_request::ModelResponse {
                content: "Fast response".to_string(),
                tool_calls: Vec::new(),
                finish_reason: boomai_daemon::core::model_request::FinishReason::Stop,
                usage: boomai_daemon::core::model_request::Usage {
                    prompt_tokens: 5,
                    completion_tokens: 10,
                    total_tokens: 15,
                },
                model_id: boomai_daemon::core::types::ModelId("fast".to_string()),
                latency_ms: 50,
                warnings: Vec::new(),
            })
        } else {
            Err(ProviderError::new(
                ProviderErrorKind::BadRequest,
                ProviderId("fast".to_string()),
                None,
                "Wrong request",
            ))
        }
    });

    daemon.configure_provider(fast_provider.into(), "fast").await;

    // Launch concurrent requests
    let futures = vec![
        daemon.chat("Fast request 1".into()),
        daemon.chat("Fast request 2".into()),
        daemon.chat("Other request".into()), // This should fail
    ];

    let results = futures::future::join_all(futures).await;

    // Two should succeed, one should fail
    let success_count = results.iter().filter(|r| r.is_success()).count();
    let failure_count = results.iter().filter(|r| !r.is_success()).count();

    assert_eq!(success_count, 2);
    assert_eq!(failure_count, 1);
}
