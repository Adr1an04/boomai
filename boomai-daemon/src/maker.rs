use crate::core::{
    model_request::{ModelRequest, RequestPriority, TruncationPolicy},
    provider_runner::ProviderRunner,
    types::{Message, Role},
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

/// race n workers to first-to-k agreement
#[allow(dead_code)]
pub async fn race_to_k(
    provider: Arc<ProviderRunner>,
    prompt: String,
    n: usize,
    k: usize,
    cancellation: CancellationToken,
) -> String {
    let prompt_arc: Arc<str> = Arc::from(prompt);
    let mut set = JoinSet::new();
    let mut votes: HashMap<String, usize> = HashMap::new();

    for _ in 0..n {
        let p = provider.clone();
        let pr = prompt_arc.clone();
        let cancel_token = cancellation.clone();

        set.spawn(async move {
            let req = ModelRequest {
                messages: vec![Message { role: Role::User, content: pr.to_string() }],
                tools: Vec::new(),
                response_format: None,
                max_output_tokens: None,
                temperature: None,
                top_p: None,
                stop: Vec::new(),
                seed: None,
                stream: false,
                tags: Vec::new(),
                priority: RequestPriority::Background,
                hard_deadline_ms: None,
                require_json: false,
                truncation: TruncationPolicy::ErrorIfTooLarge,
            };

            // Check if already cancelled before starting
            if cancel_token.is_cancelled() {
                return Err(crate::core::provider_error::ProviderError::new(
                    crate::core::provider_error::ProviderErrorKind::Cancelled,
                    crate::core::provider_error::ProviderId("cancelled".to_string()),
                    None,
                    "Request was cancelled",
                ));
            }

            p.execute(req).await
        });
    }

    while let Some(res) = set.join_next().await {
        // Check if we should cancel due to external cancellation
        if cancellation.is_cancelled() {
            set.abort_all();
            return String::new(); // Return empty on cancellation
        }

        if let Ok(Ok(response)) = res {
            let content = response.content.trim().to_string();
            if content.is_empty() || content.len() > 1000 {
                continue;
            }
            let count = votes.entry(content.clone()).or_insert(0);
            *count += 1;
            if *count >= k {
                // Cancel remaining tasks
                cancellation.cancel();
                set.abort_all();
                return content;
            }
        }
    }

    votes.into_iter().max_by_key(|(_, v)| *v).map(|(k, _)| k).unwrap_or_default()
}
