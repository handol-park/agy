# Curriculum Roadmap

This roadmap defines how this repository evolves from a minimal loop into a small, testable, and extensible Rust agent framework.

## Outcome
Build an agent framework with:
- stable and explicit core abstractions
- deterministic default tests
- isolated provider/tool boundaries
- replayable execution traces
- staged design docs that capture architectural decisions

## What Is an Agent? (Evolving Definition)
In this repository, an "agent" is defined by capabilities added over time.

- `001`: goal-directed closed loop (perceive, decide, act, feedback, stop)
- `002`: interchangeable language-model adapter for synthesis
- `003`: typed memory state across steps
- `004`: structured and validated action space
- `005`: tool execution interface and registry
- `006`: planner module decoupled from runtime
- `007`: prompt/policy constraints for safe behavior
- `008`: production runtime controls and telemetry
- `009`: measurable benchmark and regression framework
- `010`: packaged developer-facing framework

## Current Status

## 000. Curriculum and Tracking
- [x] Define staged curriculum and deliverables.
- [ ] Keep stage status updated as work lands.
- Design doc: `docs/design/000-curriculum-roadmap.md`

## 001. Core Loop Foundations
- [x] Implement bounded loop.
- [x] Return run state and per-step trace.
- [x] Provide CLI adapter around library core.
- Design doc: `docs/design/001-minimal-agent-loop.md`

## 002. LLM Adapter Layer
- [x] Add model client abstraction and mock implementation.
- [x] Support one real provider behind adapter boundary.
- [x] Keep deterministic tests via mocks.
- Design doc: `docs/design/002-llm-adapter-layer.md`

## Stage Order and Dependencies
- `003` depends on `001` and `002`.
- `004` depends on `003`.
- `005` depends on `004`.
- `006` depends on `003`, `004`, and `005`.
- `007` depends on `004`, `005`, and `006`.
- `008` depends on all runtime-facing stages (`003` through `007`).
- `009` depends on stable behavior from `004` through `008`.
- `010` depends on stable APIs and docs from all prior stages.

## 003. Typed Memory Model
- Status: [ ] planned
- Design doc: `docs/design/003-memory-model.md`
- Scope:
- replace raw transcript strings with typed memory structs
- make state transitions explicit for each observe step
- serialize memory and traces for replay/debugging
- Proposed interfaces:
- `Memory` struct with goal, observations, derived facts, and status flags
- `Observation` enum for user/tool/runtime inputs
- `MemoryUpdate` helpers to mutate state deterministically
- Tests:
- memory initialization and update sequencing
- round-trip serialization/deserialization
- regression for "empty observation" and repeated updates
- Exit criteria:
- `Agent` loop no longer depends on `VecDeque<String>`
- replay artifact can reproduce final memory and run outcome

## 004. Action Model
- Status: [ ] planned
- Design doc: `docs/design/004-action-model.md`
- Scope:
- expand to structured actions: `AskUser`, `Finish`, `CallTool`, `Reflect` (optional)
- validate action payloads before execution
- separate planning output from execution side effects
- Proposed interfaces:
- `Action` enum with typed payload structs
- `ActionValidationError` for unsupported/invalid actions
- `ActionResult` to feed observe/memory updates
- Tests:
- valid action execution paths
- invalid payload and unsupported action handling
- backward compatibility for existing `AskUser`/`Finish` behavior
- Exit criteria:
- action execution and state mutation are distinct steps
- planner output is machine-validated before runtime execution

## 005. Tool Interface and Registry
- Status: [ ] planned
- Design doc: `docs/design/005-tool-interface-and-registry.md`
- Scope:
- define generic `Tool` trait and registry lookup
- implement at least two deterministic local tools
- establish typed tool result and error model
- Proposed interfaces:
- `Tool` trait: metadata, schema, execute
- `ToolRegistry` for lookup and dispatch
- `ToolCall` and `ToolOutput` typed payloads
- Tests:
- registry lookup success/failure
- tool execution success/error
- deterministic behavior with fixed inputs
- Exit criteria:
- runtime can execute `Action::CallTool` through registry only
- no direct tool-specific branching in agent core loop

## 006. Planner Architecture
- Status: [ ] planned
- Design doc: `docs/design/006-planner-architecture.md`
- Scope:
- move decision logic out of `Agent` runtime into planner module
- allow rule-based and model-based planners behind one trait
- keep planner inputs purely typed (`Memory`, available tools, budget)
- Proposed interfaces:
- `Planner` trait: `plan_next(...) -> Action`
- `RulePlanner` baseline deterministic implementation
- `ModelPlanner` adapter-driven implementation
- Tests:
- deterministic planner fixtures for known memory snapshots
- planner fallback behavior on model errors
- no-op/finish decisions under exhausted budgets
- Exit criteria:
- runtime orchestrates only loop mechanics and execution
- planner logic is unit-testable without environment side effects

## 007. Prompt and Policy System
- Status: [ ] planned
- Design doc: `docs/design/007-prompt-and-policy-design.md`
- Scope:
- version prompt templates and planner instruction strategy
- add policy guardrails before tool execution and before user-visible finalization
- implement refusal/restriction paths with explicit reasons
- Proposed interfaces:
- `PromptTemplate` and `PromptVersion`
- `PolicyEngine` with check methods returning allow/deny + reason
- policy events included in trace output
- Tests:
- policy allow/deny decisions for representative scenarios
- prompt rendering snapshot tests
- refusal behavior regression tests
- Exit criteria:
- policy is enforced centrally, not ad-hoc in tools/planner
- prompt changes are versioned and test-covered

## 008. Runtime and Observability
- Status: [ ] planned
- Design doc: `docs/design/008-runtime-and-observability.md`
- Scope:
- add retries, timeouts, cancellation, and step/failure budgets
- emit structured runtime events and timings
- improve error taxonomy for actionable diagnostics
- Proposed interfaces:
- `RunConfig` for budgets/timeouts/retry policy
- `RunEvent` typed stream for step/action/tool/model events
- `RunError` categories (policy, planner, tool, model, runtime)
- Tests:
- timeout and cancellation behavior
- retry behavior with deterministic transient failures
- telemetry event coverage for normal and failure paths
- Exit criteria:
- runtime failure modes are bounded and observable
- event stream is sufficient to debug failed runs without reproduction

## 009. Evaluation Framework
- Status: [ ] planned
- Design doc: `docs/design/009-evaluation-framework.md`
- Scope:
- define benchmark task set with expected outcomes
- implement regression suite and metric collection
- track latency, step count, success rate, and failure reasons
- Proposed interfaces:
- `BenchmarkCase`, `BenchmarkResult`, `MetricsSummary`
- runner for batch execution over planner/runtime configs
- baseline snapshots committed to repository
- Tests:
- benchmark harness determinism and fixture loading
- metric calculations and summary formatting
- regression detection on known failing cases
- Exit criteria:
- repository includes repeatable benchmark command
- changes can be compared against stored baseline metrics

## 010. Packaging and DX
- Status: [ ] planned
- Design doc: `docs/design/010-release-and-developer-experience.md`
- Scope:
- finalize public API boundaries and crate layout
- improve CLI UX and config loading
- provide end-to-end examples and contributor docs
- Proposed deliverables:
- stable `src/lib.rs` exports and module organization
- `examples/` for offline/mock and real-provider execution
- setup docs for local development, testing, and troubleshooting
- Tests:
- smoke tests for CLI paths
- doc-tested examples where practical
- compatibility checks for config parsing
- Exit criteria:
- new contributor can clone, run, test, and execute examples quickly
- framework API surface is intentionally versioned and documented

## Standard Stage Deliverables
Each stage should produce:
- one design doc in `docs/design/`
- implementation changes in `src/`
- tests covering new behavior
- at least one runnable example or demo path

## Suggested Implementation Cadence
- complete one stage at a time with no hidden cross-stage work
- for each stage: design doc first, then interfaces, then runtime wiring, then tests
- do not advance stage status until tests and docs both land
