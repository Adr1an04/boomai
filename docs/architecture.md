# Architecture

This document targets contributors who want to extend or harden the system. Boomai is a “lazy-use,” local-first orchestration engine that applies a hybrid MAKER approach (Maximal Decomposition, k-threshold voting, red-flagging) on top of Rust/Tokio concurrency.

## Components
- **Daemon (`boomai-daemon`)**: Axum + Tokio HTTP service. Houses orchestration logic, model abstraction, and persistence.
- **Core module (`src/core`)**: Shared types, agent trait, and model providers (HttpProvider, DummyProvider). Replaces the former `boomai-core` crate.
- **Agents (`src/agents`)**:
  - Classifier, Decomposer, Router, Verifier, Calculator, Interrogator
  - Voting and Red-Flag filters
  - Orchestrator implementing MDAP/MAKER flows with tiered reliability lanes (deterministic internal stubs + Maker voting tool)
- **Local Model Manager (`local.rs`)**: Manages Ollama-based model installs and metadata.
- **MCP (`src/mcp`)**: Client/manager scaffolding for external tool servers with JSON-RPC.
- **Persistence (`config_persistence.rs`)**: Async config load/save with history and rollback support.

## Data Model (core/types)
- `Message { role, content }`
- `ChatRequest { messages }`
- `ChatResponse { message, status, maker_context }`
- `ExecutionStatus` includes: Classifying, Decomposing, Voting { round }, ToolCall { tool }, Done, Failed, etc.
- `ModelConfig { base_url, api_key, model }` with validation
- Local model structs: `AvailableLocalModel`, `InstalledLocalModel`

## Model Provider Abstraction
- Trait: `ModelProvider::chat(ChatRequest) -> ChatResponse`
- Implementations:
  - `HttpProvider`: OpenAI-style `/chat/completions` over HTTP; optional Bearer auth
  - `DummyProvider`: echo/testing
- Swappable at runtime via shared `Arc<RwLock<Arc<dyn ModelProvider>>>`.

## Orchestration (Hybrid MAKER / MDAP)
- **Small, Isolated Steps (m=1)**: Decompose tasks into atomic actions (Decomposer) to reduce drift.
- **Parallel Consensus (ahead-by-k)**: Spawn concurrent candidates; the Maker tool waits for a lead by k before selecting a winner.
- **Structural Validation**: Red-Flag filter for pathological outputs; status gating for tool calls and verification.
- Flows:
  - SIMPLE → deterministic internal stubs (calculator/time) or single-probe
  - TOOL → Router decides tool use via the tool registry
  - COMPLEX → Full MDAP loop with classification, decomposition, routing, verification, Maker voting for reasoning
- Context isolation: minimal context passed per step to limit drift.

## Concurrency
- Tokio runtime; Axum for HTTP.
- Agents share model provider behind `RwLock<Arc<dyn ModelProvider>>`.
- Parallel candidate spawning uses `tokio::spawn` and aggregate voting.

## Persistence & Config
- `config_persistence.rs`:
  - Load/save `DaemonConfigStore` asynchronously.
  - History + rollback support.
  - Validation via `ModelConfig::validate()`.

## Local Models
- Ollama-focused install/uninstall and listing.
- Recommendations in `system.rs` based on RAM/arch.

## MCP (Tools)
- `mcp/client.rs`: JSON-RPC over child process stdio; graceful kill on drop.
- `mcp/manager.rs`: registry of clients; ready for tool discovery/listing.

## Error Handling & Resilience
- Circuit breaker and health-triggered recovery are planned (not yet implemented).
- Graceful shutdown hook is in place for the daemon; MCP clients best-effort kill on drop.

## Frontend Contract (high level)
- HTTP API on `localhost:3030`
- Endpoints: chat, config/model (save/test/reload/rollback), config/local (available/installed/install/uninstall), system profile/recommendation, MCP server/tool management.

