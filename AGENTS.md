# Repository Guidelines

## Project Structure & Module Organization
This repository is for learning agent frameworks by building one from scratch. Keep code small, inspectable, and modular.

Suggested layout as the project grows:

- `src/agent/` core loop, planner, executor modules
- `src/tools/` tool traits and implementations
- `src/memory/` short/long-term memory components
- `src/prompts/` prompt templates and policies
- `src/lib.rs` shared framework APIs
- `src/main.rs` local CLI/demo entrypoint
- `tests/` integration tests (`tests/agent_flow.rs`, etc.)
- `examples/` runnable demos showing end-to-end behavior
- `docs/` architecture notes, experiments, and lessons learned

## Build, Test, and Development Commands
Use `cargo` directly; add a `Makefile` only if it simplifies repeated workflows.

Recommended baseline:

- `cargo check` fast compile validation
- `cargo test` run unit + integration tests
- `cargo fmt --all` format code
- `cargo clippy --all-targets --all-features -D warnings` lint with warnings as errors
- `cargo run --bin agy` run local agent CLI (adjust binary name as needed)

Keep `README.md` command examples synchronized with these commands.

## Coding Style & Naming Conventions
Use 4-space indentation and UTF-8 text files. Prefer explicit names:

- Modules/files/functions/variables: `snake_case`
- Structs/enums/traits: `PascalCase`
- Constants/statics: `UPPER_SNAKE_CASE`

Prefer small traits and explicit error types (`thiserror`/`anyhow` by layer). Run `fmt` and `clippy` before every PR.

## Design Principle: Simplicity First
Treat unnecessary complexity as tech debt.

- Prefer the simplest design that satisfies the current stage.
- Avoid speculative abstractions that are not yet required.
- Add a new type or layer only when it removes real duplication or enables necessary tests/behavior.

## Testing Guidelines
Keep unit tests near modules (`mod tests`) and cross-module integration tests in `tests/`. For each framework component (planner, memory, tools), add:

- happy-path behavior test
- failure or edge-case test
- regression test for fixed bugs

Prefer deterministic tests: mock model/tool boundaries, pin fixtures, and avoid network calls in default test runs.

## Commit & Pull Request Guidelines
Use a consistent commit format from the start:

- Commit subject: imperative, concise (for example, `Add tool registry interface`)
- Optional body: motivation, approach, and side effects

PRs should include:

- summary of changes and rationale
- testing performed (for example, `cargo test`, `cargo clippy`)
- linked issue/ticket (if available)
- sample input/output logs for behavior changes

## Security & Configuration Tips
Never commit secrets. Keep local overrides in ignored files (`.env.local`) and provide `.env.example` for required variables.
