# Setup and Running

Quick path to a working local stack (daemon + desktop) for a local-first, “lazy use” workflow.

## Requirements
- Rust (stable toolchain)
- Node.js (for the desktop client)
- Ollama (for local AI models)

## Install Ollama
- **macOS**: `brew install ollama`
- **Windows**: Download installer from [ollama.ai](https://ollama.ai) and run `OllamaSetup.exe`
- **Linux**:
  ```bash
  curl -fsSL https://ollama.com/install.sh | sh
  ollama -v
  ```

Start the service and pull a model:
```bash
ollama serve
ollama pull llama2:7b   # or any model you prefer
```

## Run the backend daemon
```bash
export BOOMAI_PORT=3030
cargo run -p boomai-daemon
```

## Run the desktop app
```bash
cd desktop
npm install
npm run tauri dev
```

The desktop client will auto-connect to `localhost:3030`.

