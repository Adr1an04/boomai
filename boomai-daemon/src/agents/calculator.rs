use boomai_core::{Agent, AgentContext, ChatRequest, ChatResponse, Message, Role};
use async_trait::async_trait;
use std::sync::{Arc, RwLock};
use evalexpr::eval;

pub struct CalculatorAgent {
    model_provider: Arc<RwLock<Arc<dyn boomai_core::ModelProvider>>>,
}

impl CalculatorAgent {
    pub fn new(model_provider: Arc<RwLock<Arc<dyn boomai_core::ModelProvider>>>) -> Self {
        Self { model_provider }
    }

    fn extract_and_compute(&self, input: &str) -> Option<String> {
        // 1. Try direct evaluation
        if let Ok(val) = eval(input) {
            return Some(val.to_string());
        }

        // 2. Try simple cleaning (remove non-math chars except space)
        let cleaned: String = input.chars()
            .filter(|c| c.is_digit(10) || "+-*/(). ".contains(*c))
            .collect();
        
        if let Ok(val) = eval(&cleaned) {
            return Some(val.to_string());
        }

        None
    }
}

#[async_trait]
impl Agent for CalculatorAgent {
    async fn handle_chat(&self, req: ChatRequest, _ctx: AgentContext) -> anyhow::Result<ChatResponse> {
        let last_message = req.messages.last()
            .map(|m| m.content.clone())
            .unwrap_or_default();

        println!("[CALCULATOR] Attempting deterministic math for: {}", last_message);

        // Try deterministic calculation first
        if let Some(result) = self.extract_and_compute(&last_message) {
            println!("[CALCULATOR] Deterministic success: {}", result);
            return Ok(ChatResponse {
                message: Message {
                    role: Role::Assistant,
                    content: result,
                },
                status: boomai_core::types::ExecutionStatus::Done,
                maker_context: None,
            });
        }

        println!("[CALCULATOR] Deterministic failed. Falling back to LLM extraction.");

        // If direct math failed, ask LLM to extract ONLY the expression
        let mut extraction_messages = Vec::new();
        extraction_messages.push(Message {
            role: Role::System,
            content: "You are a Math Extractor. Output ONLY the mathematical expression from the user's text. Do not solve it. Do not add text.".to_string(),
        });
        extraction_messages.push(Message {
            role: Role::User,
            content: last_message,
        });

        let provider = {
            if let Ok(guard) = self.model_provider.read() {
                guard.clone()
            } else {
                return Err(anyhow::anyhow!("Failed to acquire read lock on model provider"));
            }
        };

        let extract_req = ChatRequest { messages: extraction_messages };
        let extract_resp = provider.chat(extract_req).await?;
        let extracted_expr = extract_resp.message.content.trim();

        println!("[CALCULATOR] Extracted expression: {}", extracted_expr);

        if let Some(result) = self.extract_and_compute(extracted_expr) {
            return Ok(ChatResponse {
                message: Message {
                    role: Role::Assistant,
                    content: result,
                },
                status: boomai_core::types::ExecutionStatus::Done,
                maker_context: None,
            });
        }

        // Final fallback: Let LLM solve it (prone to hallucination but better than error)
        println!("[CALCULATOR] Extraction failed. Fallback to LLM solve.");
        let mut messages = req.messages.clone();
        messages.insert(0, Message {
            role: Role::System,
            content: "You are a Calculator. Solve the math problem. Output ONLY the result.".to_string(),
        });

        let calc_req = ChatRequest { messages };
        let mut response = provider.chat(calc_req).await?;
        response.status = boomai_core::types::ExecutionStatus::Done;
        response.maker_context = None;
        Ok(response)
    }
}

