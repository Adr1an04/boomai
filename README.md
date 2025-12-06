# Boomai

A local AI workspace built in Rust that orchestrates multiple agents using parallel voting (First-to-ahead-by-k) to achieve complex task execution on standard hardware.

Boomai is an open-source project focused on building a privacy preserving AI operator that runs on your computer. It connects a Rust daemon with a desktop app to keep setup simple while still supporting agentic workflows and model orchestration.

<img src="docs/assets/boomai.png" alt="Boomai" width="80%">

---

## Principles

1. **Local First** – Data and orchestration stay on your machine by default.
2. **Runtime Flexibility** – Choose your own models with `ModelProvider` which is AI model agnostic and provider agnositc (works with Ollama, cloud APIs, etc.).
3. **Reliability as a Systems Problem** – The current AI architecture focuses on decomposition, validation, and error correction over brute-force model size.

---

## Architecture Overview

Boomai is built as a dual component system.

### Daemon (`boomai-daemon`)

A Rust backend that coordinates agents, manages state, indexing, and exposes an HTTP API at `localhost:3030`.

Key points:

- Built with **Axum** and **Tokio** for processing multiple operations simultaneously
- Parallel task execution to avoid sequential bottlenecks with other agents
- OpenAI style JSON over HTTP through a unified provider abstraction

### Desktop App (`desktop/`)

A cross-platform UI built with **Tauri** and **React**.

- Desktop app with native webview
- Control of chat, configuration, and tool permissions
- Auto-pairs with the running daemon, for lazy plug and play

### Mods & Tools (`mods/`)

Tooling is powered by the **Model Context Protocol (MCP)** so external services (files, productivity apps, etc.) can be integrated without bloating the daemon. Access is mediated so secrets and scopes stay protected.

---

## MAKER & MDAP: AI Agent Reliability 

Boomai applies **Massively Decomposed Agentic Processes (MDAP)** to reduce errors with smaller models.

1. **Small Action Steps** – Tasks are decomposed into small actions to limit context drift.
2. **Parallel Voting** – Multiple agents run in parallel; progress advances only when a candidate leads via voting.
3. **Structural Validation** – Outputs are checked for format and size; suspicious responses are retried before they affect downstream steps.

---

## Summary

- Simple setup with hardware-aware defaults
- Local model semantic search and retrieval for grounding
- Mod installation workflow for extending capabilities
- Rust-based indexing and orchestration for performance and safety

---

## Setup

**Requirements**

- Rust (stable toolchain)
- Node.js (for the desktop client)
- Ollama (for local AI models|temporary model provider)

**1. Install and setup Ollama**

### macOS
```bash
brew install ollama
```

### Windows
1. Visit [ollama.ai](https://ollama.ai) and click **Download for Windows**
2. Run the downloaded `OllamaSetup.exe` and follow the installer wizard
3. Once installation completes, Ollama will be available on your system

### Linux
1. Open a terminal
2. Run the official install script:
   ```bash
   curl -fsSL https://ollama.com/install.sh | sh
   ```
3. Verify installation:
   ```bash
   ollama -v
   ```

### After Installation: Start Ollama and Download a Model
```bash
# Start Ollama service
ollama serve

**2. Run the backend daemon**

```bash
# From project root
export BOOMAI_PORT=3030
cargo run -p boomai-daemon
```

**3. Run the desktop app**

```bash
cd desktop
npm install
npm run tauri dev
```

The desktop client will automatically connect to the daemon at `localhost:3030`.

---

## License

Boomai is available under the MIT License.