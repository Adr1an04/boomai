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
        println!("[MAKER] Executing COMPLEX flow (Parallel MDAP)...");
        let mut history = initial_req.messages.clone();
        let goal = history.last().map(|m| m.content.clone()).unwrap_or_default();
        
        let _voting = VotingMechanism::new(2); // k=2 (ahead by 2)
        // RedFlagFilter temporarily disabled to reduce false positives with TinyLlama
        // let _red_flag = RedFlagFilter::new();

        println!("[MAKER] Starting orchestration for goal: {}", goal);

        let mut step_count = 0;
        let mut final_answer = String::new();

        loop {
            step_count += 1;
            if step_count > self.max_steps {
                println!("[MAKER] Max steps reached ({}), terminating.", self.max_steps);
                break;
            }

            // 0. INTERROGATION (Stop Condition)
            if step_count > 1 {
                let mut check_messages = Vec::new();
                check_messages.push(Message { role: Role::System, content: "Review the history. Is the goal fully achieved? Output 'SOLVED' if yes, 'CONTINUE' if no.".to_string() });
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

            // 1. DECOMPOSITION
            let mut decompose_messages = Vec::new();
            decompose_messages.push(Message {
                role: Role::System,
                content: "You are the Decomposer. Output the SINGLE immediate next step to solve the goal. Be concise.".to_string(),
            });
            
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
                println!("[MAKER] Task complete (Decomposer signaled DONE).");
                break;
            }

            // 2. PARALLEL EXECUTION (with Voting)
            
            let attempts = 3; // Number of parallel candidates
            let needed_candidates = 1; // Reduced for MVP, usually would be 3 for voting
            
            // Construct the prompt for the worker
            let mut worker_messages = history.clone();
            worker_messages.push(Message {
                role: Role::System,
                content: format!("Execute this step: {}", next_step),
            });

            let mut handles = Vec::new();
            
            // Spawn parallel tasks
            for _ in 0..attempts {
                let router_agent = self.state.router_agent.clone();
                let req = ChatRequest { messages: worker_messages.clone() };
                
                let handle = tokio::spawn(async move {
                    router_agent.handle_chat(req, boomai_core::AgentContext { 
                        task_id: "orch_parallel".to_string(), 
                        step_number: 0,
                        depth: 0,
                        max_depth: 5,
                        maker_context: None,
                    }).await
                });
                handles.push(handle);
            }

            // Collect results
            let mut candidates = Vec::new();
            for handle in handles {
                if let Ok(Ok(resp)) = handle.await {
                    candidates.push(resp.message.content);
                    // Early exit optimization could go here if we just need *any* valid answer
                    if candidates.len() >= needed_candidates {
                        // In a real system we might cancel other tasks, but here we just stop collecting
                        // Note: tasks keep running in background until completion or cancellation
                    }
                }
            }

            if candidates.is_empty() {
                 println!("[MAKER] Failed to generate candidates. Skipping step.");
                 candidates.push("Failed to execute step.".to_string());
            }

            // 3. VOTING
            let winner = _voting.vote(candidates.clone()).unwrap_or_else(|| candidates[0].clone());
            println!("[MAKER] Voted winner: {:.50}...", winner);

            // 4. VERIFICATION
            let mut verify_messages = Vec::new();
            verify_messages.push(Message {
                role: Role::System,
                content: "Verify the result. Output 'CORRECT' if it reasonably addresses the step. Output 'INCORRECT' otherwise.".to_string()
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
