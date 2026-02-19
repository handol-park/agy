# Current Design and Implementation

## Purpose
`agy` is a minimal Rust agent prototype focused on learning core agent-loop mechanics before adding model/tool complexity.

The current design implements:
- a bounded loop (`max_steps`)
- explicit planning and action selection
- environment abstraction for interaction
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

This trait is the key decoupling point: the core loop does not depend on CLI or network APIs.

## Agent Loop
`Agent::run(goal, env)` executes:
1. Initialize `transcript`, `last_observation`, and `traces`.
2. For each step up to `max_steps`:
   - `plan(...)` generates a thought string.
   - `act(...)` chooses `AskUser` or `Finish`.
   - trace is recorded.
   - if `AskUser`, read an observation via `env.ask(...)` and append to transcript.
   - if `Finish`, return immediately with `RunState::Finished`.
3. If no finish action occurs, return `RunState::MaxStepsReached`.

Current policy is intentionally simple:
- step 1 asks for one critical constraint
- next step synthesizes constraint into a final answer

## CLI Adapter (`src/main.rs`)
`StdioEnv` implements `Environment` by delegating to `read_line`.
`main`:
- prompts for goal
- creates `Agent::new(3)`
- runs loop
- prints all `StepTrace` entries
- prints final state

## Testing
`src/lib.rs` includes unit tests with `FakeEnv`:
- successful finish after collecting one constraint
- max-step termination when budget is too small

This validates loop control flow without CLI I/O.

## Dev Environment
`flake.nix` provides a Rust dev shell with:
- `cargo`, `rustc`, `clippy`, `rustfmt`, `rust-analyzer`
- `rustPlatform.rustLibSrc` and `RUST_SRC_PATH` for rust-analyzer stdlib indexing

## Known Limits and Next Extensions
Current loop has no model calls, no tool registry, and no structured memory beyond raw transcript.

Natural next steps:
- replace string-based `plan` with structured planner output
- add `Action::CallTool` and a tool execution layer
- add typed memory state and serialization
- add integration tests for CLI traces
