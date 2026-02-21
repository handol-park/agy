# Minimal Agent Loop Design

## Purpose
`agy` is a minimal Rust agent prototype focused on learning core agent-loop mechanics with a pluggable model path.

## Agent Definition at Stage 001
At this stage, an agent is a goal-directed closed-loop system that:
- perceives observations as outcomes of deliberate actions
- plans the next action from current state
- executes that action to influence the environment
- repeats until a stop condition is met

It is intentionally minimal: one interaction to gather a key constraint, then one synthesis step.

In this repository's `001` implementation, that definition is instantiated as a bounded
`perceive -> plan -> act -> observe` loop with per-step traces and a max-step termination.

The current design implements:
- a bounded loop (`max_steps`)
- asynchronous execution as the default runtime model
- explicit perceive/plan/act/observe phases
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
- `AgentMemory` (internal): loop state with:
  - transcript (constraint history)
  - last observation
- `ActionOutcome` (internal):
  - `Observation(String)`
  - `Finished(String)`
- `Environment` trait: boundary for external interaction.
  - `async fn ask(&mut self, prompt: &str) -> String`
- `LanguageModel` trait: synthesis boundary.
  - `async fn synthesize(&self, goal: &str, constraint: &str) -> Result<String, String>`
- `TemplateModel`: deterministic local fallback model.

These traits are the key decoupling points: the core loop does not depend on CLI or direct HTTP code.

## Agent Loop
`Agent::run_with_model(goal, env, model).await` executes:
1. Initialize `memory` and `traces`.
2. For each step up to `max_steps`:
   - `perceive(...)` reads the latest observation from memory.
   - `plan(...)` generates a thought string and chosen action from perception + memory.
   - trace is recorded.
   - `act(...)` executes the chosen action and returns outcome.
   - `observe(...)` updates memory from that outcome.
   - if outcome is `Finished`, return immediately with `RunState::Finished`.
3. If no finish action occurs, return `RunState::MaxStepsReached`.

Current policy is intentionally simple:
- step 1 asks for one critical constraint
- next step synthesizes constraint into a final answer

## CLI Adapter (`src/main.rs`)
`StdioEnv` implements `Environment` by delegating to `read_line`.
`OpenAiCompatModel` implements `LanguageModel` using async `reqwest` against OpenAI-compatible `/chat/completions`.
`main`:
- prompts for goal
- creates `Agent::new(3)`
- selects model source:
  - real model when `OPENAI_API_KEY` is set
  - `TemplateModel` fallback otherwise
- runs loop with `run_with_model(...).await`
- prints all `StepTrace` entries
- prints final state

## Testing
`src/lib.rs` includes async unit tests (`#[tokio::test]`) with `FakeEnv` and fake model implementations:
- successful finish after collecting one constraint with model synthesis
- fallback finish when model call fails
- max-step termination when budget is too small

This validates loop control flow without CLI I/O.

## Dev Environment
`flake.nix` provides a Rust dev shell with:
- `cargo`, `rustc`, `clippy`, `rustfmt`, `rust-analyzer`
- `rustPlatform.rustLibSrc` and `RUST_SRC_PATH` for rust-analyzer stdlib indexing

The runtime uses `tokio` for async execution.

## Runtime Configuration
For real model calls in CLI:
- `LLM_PROVIDER` (optional: `openai` default, or `glm5`)
- `LLM_API_KEY` (preferred key variable)
- `LLM_BASE_URL` and `LLM_MODEL` (optional overrides)

Aliases kept for convenience:
- `OPENAI_API_KEY` / `OPENAI_BASE_URL` / `OPENAI_MODEL`
- `GLM_API_KEY` / `GLM_BASE_URL` / `GLM_MODEL`

## Known Limits and Next Extensions
Current loop supports one-question constraint gathering and one synthesis step. Memory is still an internal minimal struct, not yet a typed, serializable memory subsystem.

Natural next steps:
- add `Action::CallTool` and a tool execution layer
- add typed memory state and serialization
- add integration tests for CLI traces and model adapter error cases
