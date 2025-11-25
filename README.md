# Boomai

Boomai is local-first AI orchestration for your models, data, and tools.
It runs on your machine, connects to local or API LLMs, indexes your workspace,
and exposes everything through an extensible assistant interface and pluggable mods.

## Status

🚧 **Under Construction** 🚧
Currently pivoting core architecture to Rust.

## Repo structure

- `boomai-daemon/` – Core daemon (Rust binary, HTTP server)
- `boomai-core/` – Core logic library (Rust)
- `desktop/` – Desktop app (Tauri + React placeholder)
- `mods/` – Built-in/example mods (placeholder)
- `docs/` – Docs and design notes

## Development

### Prerequisites
- Rust (stable)

### Running the Core Daemon
```bash
cargo run -p boomai-daemon
```
Listens on `http://127.0.0.1:3030` by default.
