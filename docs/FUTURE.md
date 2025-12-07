# Future Architecture & Roadmap

Goal: evolve Boomai from an experimental prototype into a zero-error orchestration engine capable of million-step workflows on consumer hardware. The system will use the MAKER framework (maximal decomposition, parallel voting, red-flagging) while offering a configurable “Direct Execution” switch so simple workflows can bypass heavy consensus. A fully lazy frontend is a future goal: unified UX to automate local model downloads/installs, secure service auth, and one-click enablement of MCP servers.

---

## Phase 1: Concurrency Refactor (Fixing the Foundation)

Current: Orchestrator is partially parallel but still bottlenecked; parallel candidates often incur near-linear cost.

1) Async work-stealing for candidate generation
- Replace blocking loops with `tokio::spawn`/`JoinSet` for parallel candidates.
- Keep `ModelProvider` on a pooled, non-blocking HTTP client (Ollama/cloud).
- Stream results into the vote state as soon as tasks finish.

2) Red-Flag circuit breaker (pre-vote)
- Drop pathological outputs before voting:
  - Length guard (e.g., >700 tokens for atomic steps) → resample.
  - Format guard (malformed JSON) → fail fast and resample.

---

## Phase 2: MAKER Reliability Scaling + Direct Execution

Current: Majority/single-shot patterns; small models fail as context grows.

1) First-to-ahead-by-k consensus
- Race concludes when one answer leads the runner-up by k.
- Enables log-linear cost scaling Θ(s ln s).
- Hardware-aware k (higher k for weaker models).

2) Maximal decomposition (m=1)
- Enforce atomic steps in `DecomposerAgent`.
- Recursive: emit one atomic step, execute, re-evaluate.
- Context pruning: only prior result, not full failed history.

3) Direct Execution switch (future)
- Configurable bypass to skip consensus for simple, low-risk workflows.
- Route trivial tasks through a minimal, single-shot path to reduce latency and cost.

---

## Phase 3: “Lazy” Local Experience

Current: Users manually install/pull models; config is manual. Backend already exposes install/uninstall and system profile endpoints; UI automation is future work.

1) Lifecycle management endpoints
- `POST /config/local/install_model`: background download via Tokio (exists).
- `GET /system/profile`: auto-detect RAM/VRAM for recommendations (exists).
- One-click enablement of MCP servers from the UI (future).

2) Hardware-aware defaults
- <8 GB RAM: default to cloud recommended.
- 16 GB+: default to local recommended (quantized 7B class).
- Populate `AppState` from `sysinfo` at startup.

3) Frontend automation (future)
- Lazy UX handles model download/install, MCP enable, and service auth.
- Unified UI flow hides backend plumbing.

---

## Phase 4: Ecosystem (MCP & RAG)

Current: MCP scaffolded; not fully exposed to RouterAgent/UI.

1) Full MCP client
- Treat tools (filesystem, git, mail, etc.) as MCP servers.
- Daemon mediates all tool calls; secrets stay server-side.
- Single-click MCP enablement from the frontend (future).

2) Local RAG “brain”
- Background indexer to local vector store (e.g., lance/sqlite-vec).
- Expose `search_workspace` as a default tool to ground generations.

---

## Long-Term Research Directions

1) Insight vs. execution decomposition
- Apply MAKER to planning/insight generation, not only execution, with the same voting protocols.

2) Multi-model orchestration
- Dynamic routing by difficulty/cost:
  - Simple: local TinyLlama-class
  - Complex: higher-tier models (DeepSeek-R1, GPT-4o)
- Optimize reliability per dollar.

---

This roadmap is the architectural north star: prioritize reliability and system robustness over surface features. Implementation should remain local-first, privacy-preserving, and hardware-aware.

