# Boomai

Boomai is a lazy use, local-first AI orchestration engine. It targets the reliability cliff in agentic systems by combining Rust/Tokio concurrency with the a hybrid MAKER framework (Maximal Agentic Decomposition, k-threshold voting, red-flagging).

---

## The Problem: The Reliability Cliff

Long-horizon agents fail exponentially: a 1% per-step error yields ~63% failure over 100 steps. Larger models are costly, slow, and privacy-hostile. Boomai attacks the problem with architecture, not just model size. Past that it is all locally ran allowing users to use boomai locally and comfortably.

---

## The Solution: MAKER

1. **Maximal Decomposition (m=1)**  
   Break tasks into atomic, single-step units; `DecomposerAgent` isolates context to avoid drift and hallucination loops.

2. **Parallelized Consensus (first-to-ahead-by-k)**  
   Tokio spawns multiple candidates per step; voting waits until one answer leads the runner-up by k, improving statistical accuracy so smaller models can survive long horizons.

3. **Structural Red-Flagging**  
   Outputs are filtered for structural pathologies (length, malformed JSON) before voting; “sick” candidates are discarded to prevent correlated errors.

---

## System Architecture

### Daemon (`boomai-daemon`)
- Stack: Axum (HTTP), Tokio (async runtime), Tower-ready middleware.
- Concurrency: work-stealing scheduler; parallel candidate generation; shared provider behind `Arc<RwLock<Arc<dyn ModelProvider>>>`.
- Abstraction: `ModelProvider` speaks OpenAI-style JSON; hot-swap local (Ollama) or cloud backends without code changes.

### Interface (`desktop/`)
- Tauri v2 + React using native webview (WebKit/WebView2) to keep RAM free for models.
- Auto-profiles hardware (`/system/profile`) to suggest local model defaults.

### Extensibility (`mods/`)
- MCP client treats external tools (filesystem, git, etc.) as standardized attachments instead of bespoke integrations.

---

## Quick Start

Prereqs: Rust (stable), Node.js.

```bash
# Daemon
cargo run -p boomai-daemon   # listens on 127.0.0.1:3030

# Desktop (new terminal)
cd desktop
npm install && npm run tauri dev
```

For full platform steps (including Ollama install), see `docs/setup.md`.

---

## Current Status (Alpha)
- Core daemon, HTTP API, and `ModelProvider` trait are stable.
- Voting logic (first-to-ahead-by-k) implemented; red-flag filter and deterministic calculator in place.
- Concurrency: orchestrator being hardened for parallel candidate generation.
- MCP plumbing scaffolded; UI exposure pending.

Contributions welcome, especially on parallelizing `MakerOrchestrator` and expanding MCP/tool coverage.

---

Boomai is MIT-licensed and built for developers who want AI that is local, private, and reliable.

