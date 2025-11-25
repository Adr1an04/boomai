# Boomai

Boomai is an experiment in local-first AI for your own machine.

The idea is simple: a small Rust daemon that talks to language models and your files, plus a desktop client on top so you don’t have to live in a terminal. It’s meant to be something you can actually run, poke at, and hack on.

## Status

Very early.

APIs, layout, and names will change. Things will break. Expect rough edges.

Right now the work is mostly on the Rust daemon and a minimal HTTP API. Desktop and mods are just stubs on disk.

## Layout

- `boomai-daemon/` – Rust binary, process entrypoint and HTTP server
- `boomai-core/` – Rust library for core logic and types
- `desktop/` – Desktop client (Tauri + React)
- `mods/` – planned mods and examples
- `docs/` – notes, sketches, and design docs

## Development

You’ll need:

- Rust (stable toolchain)
- Node.js (for desktop)

Run the daemon from the repo root:

```bash
cargo run -p boomai-daemon
```

Run the desktop client:

```bash
cd desktop
npm install
npm run tauri dev
```