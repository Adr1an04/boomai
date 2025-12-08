use crate::agents::red_flag::RedFlagFilter;
use crate::agents::voting::VotingMechanism;
use crate::core::{Agent, AgentContext, ChatRequest, ChatResponse, ExecutionStatus, Message, Role};
use crate::state::AppState;
use std::sync::Arc;
use tracing::info;

pub struct MakerOrchestrator {
    state: Arc<AppState>,
    max_steps: usize,
}

impl MakerOrchestrator {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state, max_steps: 5 }
    }

    pub async fn run(&self, initial_req: ChatRequest) -> anyhow::Result<ChatResponse> {
        // Pull the latest user text for deterministic pre-flight routing.
        let user_text =
            initial_req.messages.last().map(|m| m.content.to_lowercase()).unwrap_or_default();

        // Hard heuristics before invoking the classifier to avoid LLM chatter.
        let is_math_like = {
            let t = user_text.as_str();
            let has_math_word = t.contains("sum")
                || t.contains("add")
                || t.contains("subtract")
                || t.contains("multiply")
                || t.contains("divide")
                || t.contains("calculate")
                || t.contains("calc ")
                || t.contains("math")
                || t.contains("solve")
                || t.contains("plus")
                || t.contains("minus")
                || t.contains("times")
                || t.contains("power")
                || t.contains("sqrt");

            // Detect arithmetic operators only when surrounded by digits (avoids "5-step" or "rust + tauri").
            let bytes = t.as_bytes();
            let mut has_digit_op_digit = false;
            for i in 0..bytes.len() {
                let b = bytes[i];
                if (b == b'+' || b == b'-' || b == b'*' || b == b'/')
                    && i > 0
                    && bytes[i - 1].is_ascii_digit()
                {
                    let mut j = i + 1;
                    while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                        j += 1;
                    }
                    if j < bytes.len() && bytes[j].is_ascii_digit() {
                        has_digit_op_digit = true;
                        break;
                    }
                }
            }

            has_math_word || has_digit_op_digit
        };

        let is_structured_plan = {
            let t = user_text.as_str();
            let has_steps_word = t.contains("step")
                || t.contains("steps")
                || t.contains("checklist")
                || t.contains("plan")
                || t.contains("guide");
            let has_number = t.chars().any(|c| c.is_ascii_digit());
            has_steps_word && has_number
        };

        let is_tool_like = user_text.trim_start().starts_with('/');

        if is_tool_like {
            println!("[MAKER] Pre-flight: tool-like request detected. Routing to TOOL flow.");
            return self.run_tool_flow(initial_req).await;
        }

        if is_math_like {
            println!("[MAKER] Pre-flight: math-like request detected. Routing to SIMPLE flow.");
            return self.run_simple_flow(initial_req).await;
        }

        if is_structured_plan {
            println!("[MAKER] Pre-flight: structured plan detected. Routing to COMPLEX flow.");
            return self.run_complex_flow(initial_req).await;
        }

        // Otherwise, fall back to classifier for routing.
        println!("[MAKER] Classifying request...");
        let class_resp =
            self.state.classifier_agent.handle_chat(initial_req.clone(), AgentContext).await?;

        let raw_classification = class_resp.message.content.trim();
        let class_token = raw_classification.split_whitespace().next().unwrap_or("").to_uppercase();

        info!(
            target: "maker",
            raw_classification = raw_classification,
            class_token = class_token,
            "classification"
        );

        match class_token.as_str() {
            "SIMPLE" if is_math_like => self.run_simple_flow(initial_req).await,
            "TOOL" => self.run_tool_flow(initial_req).await,
            "COMPLEX" => self.run_complex_flow(initial_req).await,
            // Fallback: route to router/general answer instead of calculator
            _ => self.run_tool_flow(initial_req).await,
        }
    }

    async fn run_simple_flow(&self, req: ChatRequest) -> anyhow::Result<ChatResponse> {
        info!(target: "maker", event = "simple_flow");
        self.state.calculator_agent.handle_chat(req, AgentContext).await
    }

    async fn run_tool_flow(&self, req: ChatRequest) -> anyhow::Result<ChatResponse> {
        info!(target: "maker", event = "tool_flow");
        let mut resp = self.state.router_agent.handle_chat(req, AgentContext).await?;
        resp.status = ExecutionStatus::Done;
        Ok(resp)
    }

    async fn run_complex_flow(&self, initial_req: ChatRequest) -> anyhow::Result<ChatResponse> {
        info!(target: "maker", event = "complex_flow_start");
        let mut history = initial_req.messages.clone();
        let goal = history.last().map(|m| m.content.clone()).unwrap_or_default();

        let voting = VotingMechanism::new(2); // k=2 (ahead by 2)
        let red_flag = RedFlagFilter::new();

        info!(target: "maker", event = "orchestration_start", goal = %goal);

        let mut step_count = 0;
        let mut final_answer = String::new();

        loop {
            step_count += 1;
            if step_count > self.max_steps {
                return Ok(ChatResponse {
                    message: Message {
                        role: Role::Assistant,
                        content: format!(
                            "Stopped after {} steps. Last state: {}",
                            self.max_steps, final_answer
                        ),
                    },
                    status: ExecutionStatus::Done, // or failed lol?
                    maker_context: None,
                });
            }

            if step_count > 1 {
                let mut check_messages = Vec::new();
                check_messages.push(Message {
                    role: Role::System,
                    content: "Review the history. Is the goal fully achieved?".to_string(),
                });
                check_messages.push(Message {
                    role: Role::User,
                    content: format!(
                        "Goal: {}\nHistory:\n{}",
                        goal,
                        history
                            .iter()
                            .skip(1)
                            .map(|m| format!("{:?}: {}", m.role, m.content))
                            .collect::<Vec<_>>()
                            .join("\n")
                    ),
                });

                let check_req = ChatRequest { messages: check_messages };
                let check_resp =
                    self.state.interrogator_agent.handle_chat(check_req, AgentContext).await?;

                if check_resp.message.content.to_uppercase().contains("SOLVED") {
                    println!("[MAKER] Interrogator signaled SOLVED. Stopping.");
                    break;
                }
            }

            let mut decompose_messages = Vec::new();
            decompose_messages.push(Message {
                role: Role::System,
                content: "You are the Decomposer. Your job is to determine the single immediate next step to solve the user's goal, given the history of steps taken so far. If the goal is fully achieved, output 'DONE'. Otherwise, output just the instruction for the next step.".to_string(),
            });

            decompose_messages.push(Message {
                role: Role::User,
                content: format!(
                    "Goal: {}\n\nHistory of steps taken:\n{}\n\nWhat is the next step?",
                    goal,
                    history
                        .iter()
                        .skip(1)
                        .map(|m| format!("{:?}: {}", m.role, m.content))
                        .collect::<Vec<_>>()
                        .join("\n")
                ),
            });

            let next_step_req = ChatRequest { messages: decompose_messages };
            let next_step_resp =
                self.state.decomposer_agent.handle_chat(next_step_req, AgentContext).await?;
            let next_step = next_step_resp.message.content.trim();

            info!(
                target: "maker",
                event = "step",
                step = step_count,
                next_step = %next_step
            );

            if next_step.eq_ignore_ascii_case("DONE") {
                println!("[MAKER] Task complete.");
                break;
            }

            let mut candidates = Vec::new();
            let attempts = 3; // reduced attempts
            let needed_candidates = 1; // reduced for MVP/TinyLlama

            let mut worker_messages = history.clone();
            worker_messages.push(Message {
                role: Role::System,
                content: format!("Execute this step: {}", next_step),
            });

            for _i in 0..attempts {
                if candidates.len() >= needed_candidates {
                    break;
                }

                let worker_req = ChatRequest { messages: worker_messages.clone() };
                let resp = self.state.router_agent.handle_chat(worker_req, AgentContext).await;

                if let Ok(r) = resp {
                    let content = r.message.content;

                    // Apply RedFlag filtering
                    if red_flag.is_flagged(&content) {
                        info!(target: "maker", event = "red_flag", step = step_count, candidate = %content);
                        continue;
                    }

                    candidates.push(content);
                }
            }

            if candidates.is_empty() {
                println!("[MAKER] Failed to generate candidates. Skipping step.");
                candidates.push("Failed to execute step.".to_string());
            }

            let winner = voting.vote(candidates.clone()).unwrap_or_else(|| candidates[0].clone());
            info!(
                target: "maker",
                event = "vote",
                step = step_count,
                candidates = ?candidates,
                winner = ?winner
            );

            let mut verify_messages = Vec::new();
            verify_messages.push(Message {
                role: Role::System,
                content: "You are the Verifier. Check if the following Result correctly solves the Step. If yes, output 'CORRECT'. If no, output 'INCORRECT'.".to_string()
            });
            verify_messages.push(Message {
                role: Role::User,
                content: format!("Step: {}\nResult: {}", next_step, winner),
            });

            let verify_req = ChatRequest { messages: verify_messages };
            let verify_resp =
                self.state.verifier_agent.handle_chat(verify_req, AgentContext).await?;

            if verify_resp.message.content.to_uppercase().contains("CORRECT") {
                println!("[MAKER] Step verified.");
                history.push(Message {
                    role: Role::Assistant,
                    content: format!("Step: {}\nResult: {}", next_step, winner),
                });
                final_answer = winner;
            } else {
                println!("[MAKER] Verification failed. Retrying step not implemented in MVP, proceeding with caution.");
                history.push(Message {
                    role: Role::Assistant,
                    content: format!("Step: {}\nResult: {}", next_step, winner),
                });
                final_answer = winner;
            }
        }

        Ok(ChatResponse {
            message: Message {
                role: Role::Assistant,
                content: if final_answer.is_empty() {
                    "I processed the request but generated no output.".to_string()
                } else {
                    final_answer
                },
            },
            status: ExecutionStatus::Done,
            maker_context: None,
        })
    }
}
