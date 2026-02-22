# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What Is This

A Rust agent framework built from scratch as a learning project. The agent runs a perceive → plan → act → observe loop. Development follows a staged curriculum (001–010), where each stage adds one capability with a design doc, implementation, and tests. Stages 001–005 are complete; 006–010 are planned.

## Build & Development Commands

```bash
cargo check                                              # fast compile check
cargo test                                               # all unit + integration tests
cargo test <test_name>                                   # single test by name
cargo fmt --all                                          # format
cargo clippy --all-targets --all-features -D warnings    # lint (warnings = errors)
cargo run --bin agy                                      # CLI entrypoint
cargo run --bin scorecard -- benchmarks/provider-results.example.json  # scorecard util
```

Nix dev shell available via `nix develop` (provides toolchain + rust-analyzer).

## Architecture

### Core Loop (`src/lib.rs`)

`Agent::run_with_model()` drives the loop: for each step up to `max_steps`, it calls `perceive()` → `plan()` → `act()` → `observe()`. Step 1 always asks the user for a constraint; step 2+ synthesizes via the model. The loop exits on `Action::Finish` or `MaxStepsReached`.

### Key Traits

| Trait | Location | Purpose |
|-------|----------|---------|
| `Environment` | `src/lib.rs` | I/O boundary (stdin in prod, `FakeEnv` in tests) |
| `LanguageModel` | `src/lib.rs` | LLM abstraction (`TemplateModel` deterministic fallback, `OpenAiCompatModel` for HTTP) |
| `Tool` | `src/tools/mod.rs` | Synchronous tool execution with JSON in/out |

### Module Map

- **`src/action/`** — `Action` enum (`AskUser`, `Finish`, `CallTool`), validation, `ActionResult`
- **`src/memory/`** — `Memory` struct (goal + observations + constraint history), `MemorySnapshot` for replay
- **`src/model/`** — `OpenAiCompatModel` (POST to `/chat/completions` with bearer auth)
- **`src/tools/`** — `ToolRegistry` + builtins (`CalculatorTool`, `TextSearchTool`); `default_registry()` wires them
- **`src/scorecard.rs`** — Provider benchmark formatting utility

### Data Flow

Actions are validated before execution (`action.validate()`). Tool calls dispatch through `ToolRegistry::execute()` — the core loop has no tool-specific branching. Memory is updated in the `observe()` phase with typed `Observation` records.

### Model Selection (`src/main.rs`)

Reads `LLM_PROVIDER` (openai/glm5), `LLM_API_KEY`, `LLM_BASE_URL`, `LLM_MODEL` from env. Falls back to `TemplateModel` if no key is set.

## Development Workflow

### Staged Curriculum

Each stage follows: **design doc first** → interfaces → implementation → tests. Do not advance stage status until both tests and docs land. Design docs live in `docs/design/NNN-*.md`. See `docs/design/000-curriculum-roadmap.md` for the full plan and dependency graph.

### Testing Patterns

All tests are deterministic — no network calls. Use `FakeEnv` (canned replies) and fake `LanguageModel` impls to test the loop. Tool tests use fixed JSON inputs. Tests live in `mod tests` blocks within each module.

### Design Principles (from AGENTS.md)

- Simplest design that satisfies the current stage
- No speculative abstractions
- Explicit error types over dynamic errors
- `cargo fmt` and `cargo clippy` before every commit
