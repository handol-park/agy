# agy

`agy` is a Rust learning project for building an agent framework from scratch, one stage at a time.

Current progress:
- `001` minimal closed loop (`perceive -> plan -> act -> observe`)
- `002` interchangeable LLM adapter layer
- `003` typed memory model design doc drafted

## Goals
- Keep core abstractions small and explicit
- Keep tests deterministic
- Evolve the system through versioned design docs

## Project Layout
- `src/lib.rs`: core runtime loop and interfaces
- `src/main.rs`: CLI entrypoint
- `src/bin/scorecard.rs`: scorecard utility binary
- `docs/design/`: staged architecture docs (`000`-`010`)

## Requirements
- Rust toolchain (stable)
- Optional: Nix (`flake.nix`) for reproducible dev shell

## Development Commands
- `cargo check`
- `cargo test`
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -D warnings`
- `cargo run --bin agy`

## LLM Runtime Configuration (Optional)
The CLI can call OpenAI-compatible providers when API credentials are set.

Preferred env vars:
- `LLM_PROVIDER` (`openai` or `glm5`)
- `LLM_API_KEY`
- `LLM_BASE_URL`
- `LLM_MODEL`

If no API key is present, the CLI uses the deterministic `TemplateModel` fallback.

## Design Docs
Start here for architecture and staged plan:
- `docs/design/000-curriculum-roadmap.md`
- `docs/design/001-minimal-agent-loop.md`
- `docs/design/002-llm-adapter-layer.md`
- `docs/design/003-memory-model.md`
