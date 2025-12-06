use std::sync::Arc;
use crate::core::{ChatRequest, ChatResponse, Message, Role, Agent, ExecutionStatus, AgentContext};
use crate::state::AppState;
use crate::agents::voting::VotingMechanism;
use crate::agents::red_flag::RedFlagFilter;

pub struct MakerOrchestrator {
    state: Arc<AppState>,
    max_steps: usize,
}

impl MakerOrchestrator {
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            state,
            max_steps: 5,
        }
    }

    pub async fn run(&self, initial_req: ChatRequest) -> anyhow::Result<ChatResponse> {
        println!("[MAKER] Classifying request...");
        let class_resp = self.state.classifier_agent.handle_chat(initial_req.clone(), AgentContext::default()).await?;

        let classification = class_resp.message.content.trim().to_uppercase();
        println!("[MAKER] Request classified as: {}", classification);

        if classification.contains("SIMPLE") {
            self.run_simple_flow(initial_req).await
        } else if classification.contains("TOOL") {
             self.run_tool_flow(initial_req).await
        } else {
             self.run_complex_flow(initial_req).await
        }
    }

    async fn run_simple_flow(&self, req: ChatRequest) -> anyhow::Result<ChatResponse> {
        println!("[MAKER] Executing SIMPLE flow (Calculator/Direct)...");
        self.state.calculator_agent.handle_chat(req, AgentContext::default()).await
    }

    async fn run_tool_flow(&self, req: ChatRequest) -> anyhow::Result<ChatResponse> {
        println!("[MAKER] Executing TOOL flow (Router)...");
        let mut resp = self.state.router_agent.handle_chat(req, AgentContext::default()).await?;
        resp.status = ExecutionStatus::Done;
        Ok(resp)
    }

    async fn run_complex_flow(&self, initial_req: ChatRequest) -> anyhow::Result<ChatResponse> {
        println!("[MAKER] Executing COMPLEX flow (Full MDAP)...");
        let mut history = initial_req.messages.clone();
        let goal = history.last().map(|m| m.content.clone()).unwrap_or_default();
        
        let voting = VotingMechanism::new(2); // k=2 (ahead by 2)
        let red_flag = RedFlagFilter::new();

        println!("[MAKER] Starting orchestration for goal: {}", goal);

        let mut step_count = 0;
        let mut final_answer = String::new();

        loop {
            step_count += 1;
            if step_count > self.max_steps {
                return Ok(ChatResponse {
                    message: Message {
                        role: Role::Assistant,
                        content: format!("Stopped after {} steps. Last state: {}", self.max_steps, final_answer),
                    },
                    status: ExecutionStatus::Done, // or failed lol?
                    maker_context: None,
                });
            }

            if step_count > 1 {
                let mut check_messages = Vec::new();
                check_messages.push(Message { role: Role::System, content: "Review the history. Is the goal fully achieved?".to_string() });
                check_messages.push(Message { role: Role::User, content: format!("Goal: {}\nHistory:\n{}", goal, history.iter().skip(1).map(|m| format!("{:?}: {}", m.role, m.content)).collect::<Vec<_>>().join("\n")) });
                
                let check_req = ChatRequest { messages: check_messages };
                let check_resp = self.state.interrogator_agent.handle_chat(check_req, AgentContext::default()).await?;

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
                content: format!("Goal: {}\n\nHistory of steps taken:\n{}\n\nWhat is the next step?", 
                    goal, 
                    history.iter().skip(1).map(|m| format!("{:?}: {}", m.role, m.content)).collect::<Vec<_>>().join("\n")
                ),
            });

            let next_step_req = ChatRequest { messages: decompose_messages };
            let next_step_resp = self.state.decomposer_agent.handle_chat(next_step_req, AgentContext::default()).await?;
            let next_step = next_step_resp.message.content.trim();

            println!("[MAKER] Step {}: {}", step_count, next_step);

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
                let resp = self.state.router_agent.handle_chat(worker_req, AgentContext::default()).await;
                
                if let Ok(r) = resp {
                    let content = r.message.content;

                    // Apply RedFlag filtering
                    if red_flag.is_flagged(&content) {
                        println!("[MAKER] Red flag triggered on candidate. Discarding.");
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
            println!("[MAKER] Voted winner: {:.50}...", winner);

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
            let verify_resp = self.state.verifier_agent.handle_chat(verify_req, AgentContext::default()).await?;
            
            if verify_resp.message.content.to_uppercase().contains("CORRECT") {
                println!("[MAKER] Step verified.");
                history.push(Message { role: Role::Assistant, content: format!("Step: {}\nResult: {}", next_step, winner) });
                final_answer = winner;
            } else {
                println!("[MAKER] Verification failed. Retrying step not implemented in MVP, proceeding with caution.");
                 history.push(Message { role: Role::Assistant, content: format!("Step: {}\nResult: {}", next_step, winner) });
                 final_answer = winner;
            }
        }

        Ok(ChatResponse {
            message: Message {
                role: Role::Assistant,
                content: if final_answer.is_empty() { "I processed the request but generated no output.".to_string() } else { final_answer },
            },
            status: ExecutionStatus::Done,
            maker_context: None,
        })
    }
}
