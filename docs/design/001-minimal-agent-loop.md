# Minimal Agent Loop Design

## Purpose
`agy` is a minimal Rust agent prototype focused on learning core agent-loop mechanics with a pluggable model path.

The current design implements:
- a bounded loop (`max_steps`)
- explicit planning and action selection
- environment abstraction for interaction
- language-model abstraction with fallback behavior
- deterministic tests over the loop behavior

## High-Level Architecture
The code is split into:
- `src/lib.rs`: agent core logic and testable abstractions
- `src/main.rs`: CLI adapter using standard input/output

`main` is intentionally thin. It collects the goal, runs the library loop, and prints step traces and the final outcome.

## Core Types (`src/lib.rs`)
- `Agent { max_steps }`: configuration for loop budget.
- `Action`: next operation chosen by the policy.
  - `AskUser(String)`
  - `Finish(String)`
- `StepTrace`: per-step debug record with `step`, `thought`, and `action`.
- `RunState`: terminal status.
  - `Finished(String)`
  - `MaxStepsReached`
- `Environment` trait: boundary for external interaction.
  - `fn ask(&mut self, prompt: &str) -> String`
- `LanguageModel` trait: synthesis boundary.
  - `fn synthesize(&self, goal: &str, constraint: &str) -> Result<String, String>`
- `TemplateModel`: deterministic local fallback model.

These traits are the key decoupling points: the core loop does not depend on CLI or direct HTTP code.

## Agent Loop
`Agent::run_with_model(goal, env, model)` executes:
1. Initialize `transcript`, `last_observation`, and `traces`.
2. For each step up to `max_steps`:
   - `plan(...)` generates a thought string.
   - `act(...)` chooses `AskUser` or `Finish` (uses `model` for synthesis after a constraint is collected).
   - trace is recorded.
   - if `AskUser`, read an observation via `env.ask(...)` and append to transcript.
   - if `Finish`, return immediately with `RunState::Finished`.
3. If no finish action occurs, return `RunState::MaxStepsReached`.

Current policy is intentionally simple:
- step 1 asks for one critical constraint
- next step synthesizes constraint into a final answer

## CLI Adapter (`src/main.rs`)
`StdioEnv` implements `Environment` by delegating to `read_line`.
`OpenAiCompatModel` implements `LanguageModel` using OpenAI-compatible `/chat/completions`.
`main`:
- prompts for goal
- creates `Agent::new(3)`
- selects model source:
  - real model when `OPENAI_API_KEY` is set
  - `TemplateModel` fallback otherwise
- runs loop with `run_with_model(...)`
- prints all `StepTrace` entries
- prints final state

## Testing
`src/lib.rs` includes unit tests with `FakeEnv` and fake model implementations:
- successful finish after collecting one constraint with model synthesis
- fallback finish when model call fails
- max-step termination when budget is too small

This validates loop control flow without CLI I/O.

## Dev Environment
`flake.nix` provides a Rust dev shell with:
- `cargo`, `rustc`, `clippy`, `rustfmt`, `rust-analyzer`
- `rustPlatform.rustLibSrc` and `RUST_SRC_PATH` for rust-analyzer stdlib indexing

## Runtime Configuration
For real model calls in CLI:
- `OPENAI_API_KEY` (required)
- `OPENAI_BASE_URL` (optional, default `https://api.openai.com/v1`)
- `OPENAI_MODEL` (optional, default `gpt-4o-mini`)

## Known Limits and Next Extensions
Current loop supports one-question constraint gathering and one synthesis step. It still has no tool calls and no typed memory model.

Natural next steps:
- add `Action::CallTool` and a tool execution layer
- add typed memory state and serialization
- add integration tests for CLI traces and model adapter error cases
