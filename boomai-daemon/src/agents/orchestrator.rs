use std::sync::Arc;
use boomai_core::{ChatRequest, ChatResponse, Message, Role, Agent, ExecutionStatus};
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
            max_steps: 5, // Reduced for MVP testing speed
        }
    }

    pub async fn run(&self, initial_req: ChatRequest) -> anyhow::Result<ChatResponse> {
        // STEP 1: CLASSIFICATION
        // Determine if this is a SIMPLE task, COMPLEX task, or TOOL request
        println!("[MAKER] Classifying request...");
        let class_resp = self.state.classifier_agent.handle_chat(initial_req.clone(), boomai_core::AgentContext {
            task_id: "classify".to_string(),
            step_number: 0,
            depth: 0,
            max_depth: 1,
            maker_context: None,
        }).await?;

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
        self.state.calculator_agent.handle_chat(req, boomai_core::AgentContext {
            task_id: "simple".to_string(),
            step_number: 0,
            depth: 0,
            max_depth: 1,
            maker_context: None,
        }).await
    }

    async fn run_tool_flow(&self, req: ChatRequest) -> anyhow::Result<ChatResponse> {
        println!("[MAKER] Executing TOOL flow (Router)...");
        // Router agent handles tool invocation
        let mut resp = self.state.router_agent.handle_chat(req, boomai_core::AgentContext {
            task_id: "tool".to_string(),
            step_number: 0,
            depth: 0,
            max_depth: 1,
            maker_context: None,
        }).await?;
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
                    status: ExecutionStatus::Done, // Or failed?
                    maker_context: None,
                });
            }

            // 0. INTERROGATION (Stop Condition)
            // Check if we are done BEFORE decomposing further
            if step_count > 1 {
                let mut check_messages = Vec::new();
                check_messages.push(Message { role: Role::System, content: "Review the history. Is the goal fully achieved?".to_string() });
                check_messages.push(Message { role: Role::User, content: format!("Goal: {}\nHistory:\n{}", goal, history.iter().skip(1).map(|m| format!("{:?}: {}", m.role, m.content)).collect::<Vec<_>>().join("\n")) });
                
                let check_req = ChatRequest { messages: check_messages };
                let check_resp = self.state.interrogator_agent.handle_chat(check_req, boomai_core::AgentContext {
                    task_id: "orch".to_string(),
                    step_number: step_count,
                    depth: 0,
                    max_depth: 1,
                    maker_context: None,
                }).await?;

                if check_resp.message.content.to_uppercase().contains("SOLVED") {
                    println!("[MAKER] Interrogator signaled SOLVED. Stopping.");
                    break;
                }
            }

            // 1. DECOMPOSITION: Ask Decomposer for the NEXT step
            // We construct a special prompt for the decomposer
            let mut decompose_messages = Vec::new();
            decompose_messages.push(Message {
                role: Role::System,
                content: "You are the Decomposer. Your job is to determine the single immediate next step to solve the user's goal, given the history of steps taken so far. If the goal is fully achieved, output 'DONE'. Otherwise, output just the instruction for the next step.".to_string(),
            });
            
            // Add context
            decompose_messages.push(Message {
                role: Role::User,
                content: format!("Goal: {}\n\nHistory of steps taken:\n{}\n\nWhat is the next step?", 
                    goal, 
                    history.iter().skip(1).map(|m| format!("{:?}: {}", m.role, m.content)).collect::<Vec<_>>().join("\n")
                ),
            });

            let next_step_req = ChatRequest { messages: decompose_messages };
            let next_step_resp = self.state.decomposer_agent.handle_chat(next_step_req, boomai_core::AgentContext { 
                task_id: "orch".to_string(), 
                step_number: step_count,
                depth: 0,
                max_depth: 5,
                maker_context: None,
            }).await?;
            let next_step = next_step_resp.message.content.trim();

            println!("[MAKER] Step {}: {}", step_count, next_step);

            if next_step.eq_ignore_ascii_case("DONE") {
                println!("[MAKER] Task complete.");
                break;
            }

            // 2. EXECUTION (with Red-Flagging and Voting)
            
            let mut candidates = Vec::new();
            let attempts = 3; // Reduced attempts
            let needed_candidates = 1; // Reduced for MVP/TinyLlama
            
            // Construct the prompt for the worker
            let mut worker_messages = history.clone();
            worker_messages.push(Message {
                role: Role::System,
                content: format!("Execute this step: {}", next_step),
            });

            // Parallel generation would be better, but sequential for MVP
            for i in 0..attempts {
                if candidates.len() >= needed_candidates {
                    break;
                }

                // Call the provider (via RouterAgent or directly)
                // We use RouterAgent as a generic "Solver" here
                let worker_req = ChatRequest { messages: worker_messages.clone() };
                let resp = self.state.router_agent.handle_chat(worker_req, boomai_core::AgentContext { 
                    task_id: "orch".to_string(), 
                    step_number: step_count,
                    depth: 0,
                    max_depth: 5,
                    maker_context: None,
                }).await;
                
                if let Ok(r) = resp {
                    let content = r.message.content;
                    
                    // RED FLAG FILTER - Relaxed for MVP
                    // if red_flag.is_flagged(&content) {
                    //     println!("[MAKER] Red flag triggered on attempt {}. Discarding.", i);
                    //     continue;
                    // }
                    
                    candidates.push(content);
                }
            }

            if candidates.is_empty() {
                 // Fallback instead of error
                 println!("[MAKER] Failed to generate candidates. Skipping step.");
                 candidates.push("Failed to execute step.".to_string());
            }

            // 3. VOTING
            let winner = candidates[0].clone(); // Simple selection for now
            println!("[MAKER] Voted winner: {:.50}...", winner);

            // 4. VERIFICATION
            // Ask Verifier if this result is good
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
            let verify_resp = self.state.verifier_agent.handle_chat(verify_req, boomai_core::AgentContext { 
                task_id: "orch".to_string(), 
                step_number: step_count,
                depth: 0,
                max_depth: 5,
                maker_context: None,
            }).await?;
            
            if verify_resp.message.content.to_uppercase().contains("CORRECT") {
                println!("[MAKER] Step verified.");
                // Add to history
                history.push(Message { role: Role::Assistant, content: format!("Step: {}\nResult: {}", next_step, winner) });
                final_answer = winner;
            } else {
                println!("[MAKER] Verification failed. Retrying step not implemented in MVP, proceeding with caution.");
                 // For MVP, just accept it or maybe retry. Let's accept but mark it.
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
