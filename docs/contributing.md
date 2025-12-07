# Contributing Guide

## Ways to Help
- Harden hybrid MAKER flows (decomposition, ahead-by-k voting, red-flagging).
- Parallelize orchestrator loops (Tokio `spawn` paths) and reduce contention.
- Add providers (LM Studio, cloud backends) via the `ModelProvider` abstraction.
- Strengthen MCP tooling and deterministic tool coverage.
- Extend local model management (status, runtime control, better Ollama UX).

## Development Workflow
1. Install Rust + Node + Ollama (see `docs/setup.md`).
2. Run the daemon: `cargo run -p boomai-daemon`.
3. Run the desktop: `cd desktop && npm run tauri dev`.
4. Use `cargo check` before sending changes.

## Style Notes (Rust)
- Prefer async-friendly patterns (Tokio).
- Use `anyhow` for error propagation; surface clear context.
- Keep shared types in `src/core`; avoid cross-crate sprawl.
- Keep agent prompts concise; avoid brittle formatting; align with MAKER steps.

## Testing Targets
- `cargo check -p boomai-daemon`
- (Future) Integration tests for chat/config endpoints.
- Manual: start daemon + desktop and run sample chats.

## Issues & PRs
- Open an issue for any behavior regression or reliability gap.
- Small PRs are easier to review; include a short rationale and testing notes.

