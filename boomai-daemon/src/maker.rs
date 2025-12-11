use crate::core::{
    provider::ModelProvider,
    types::{ChatRequest, Message, Role},
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::task::JoinSet;

/// race n workers to first-to-k agreement
#[allow(dead_code)]
pub async fn race_to_k(
    provider: Arc<dyn ModelProvider>,
    prompt: String,
    n: usize,
    k: usize,
) -> String {
    let prompt_arc: Arc<str> = Arc::from(prompt);
    let mut set = JoinSet::new();
    let mut votes: HashMap<String, usize> = HashMap::new();

    for _ in 0..n {
        let p = provider.clone();
        let pr = prompt_arc.clone();
        set.spawn(async move {
            let req = ChatRequest::builder()
                .messages(vec![Message { role: Role::User, content: pr.to_string() }])
                .build();
            p.chat(req).await
        });
    }

    while let Some(res) = set.join_next().await {
        if let Ok(Ok(response)) = res {
            let content = response.message.content.trim().to_string();
            if content.is_empty() || content.len() > 1000 {
                continue;
            }
            let count = votes.entry(content.clone()).or_insert(0);
            *count += 1;
            if *count >= k {
                set.abort_all();
                return content;
            }
        }
    }

    votes.into_iter().max_by_key(|(_, v)| *v).map(|(k, _)| k).unwrap_or_default()
}
