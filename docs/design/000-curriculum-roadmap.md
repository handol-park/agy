# Curriculum Roadmap

This roadmap defines how this repository will evolve from a minimal loop into a full Rust agent framework.

## Outcome
Build a small, testable, and extensible agent framework with:
- clear core abstractions
- deterministic test coverage
- versioned design docs for each major architecture decision

## Stage Plan

## 000. Curriculum and Tracking
- [x] Define staged curriculum and deliverables.
- [ ] Keep stage status updated as work lands.
- Design doc: `docs/design/000-curriculum-roadmap.md`

## 001. Core Loop Foundations
- [x] Implement bounded `plan -> act -> observe` loop.
- [x] Return structured run state and per-step trace.
- [x] Provide CLI adapter around library core.
- Design doc: `docs/design/001-minimal-agent-loop.md`

## 002. LLM Adapter Layer
- [x] Add model client abstraction and mock implementation.
- [x] Support one real provider behind adapter boundary.
- [x] Keep deterministic tests via mocks.
- Design doc: `docs/design/002-llm-adapter-layer.md`

## 003. Typed Memory Model
- [ ] Replace raw transcript strings with typed memory.
- [ ] Add serialization format for replay/debugging.
- [ ] Add tests for memory update behavior.
- Design doc: `docs/design/003-memory-model.md`

## 004. Action Model
- [ ] Expand action system (`AskUser`, `Finish`, `CallTool`).
- [ ] Add schema validation for action payloads.
- [ ] Add tests for invalid/unsupported actions.
- Design doc: `docs/design/004-action-model.md`

## 005. Tool Interface and Registry
- [ ] Define `Tool` trait and registry lookup.
- [ ] Implement at least 2 local tools (e.g., calculator, grep/search).
- [ ] Add tool execution/result error model.
- Design doc: `docs/design/005-tool-interface-and-registry.md`

## 006. Planner Architecture
- [ ] Introduce `Planner` trait and rule-based planner module.
- [ ] Decouple planner from runtime loop.
- [ ] Add planner unit tests with fixtures.
- Design doc: `docs/design/006-planner-architecture.md`

## 007. Prompt and Policy System
- [ ] Version prompt templates.
- [ ] Add policy checks before tool execution.
- [ ] Add tests for policy violations/refusals.
- Design doc: `docs/design/007-prompt-and-policy-design.md`

## 008. Runtime and Observability
- [ ] Add retry/timeouts/cancellation.
- [ ] Add structured logs and event records.
- [ ] Add step budget and failure diagnostics.
- Design doc: `docs/design/008-runtime-and-observability.md`

## 009. Evaluation Framework
- [ ] Create benchmark task set and success criteria.
- [ ] Add regression suite for known scenarios.
- [ ] Track latency/step-count/success metrics.
- Design doc: `docs/design/009-evaluation-framework.md`

## 010. Packaging and DX
- [ ] Improve CLI UX and config loading.
- [ ] Add end-to-end examples in `examples/`.
- [ ] Finalize contributor/developer docs.
- Design doc: `docs/design/010-release-and-developer-experience.md`

## Standard Stage Deliverables
Each stage should produce:
- one design doc in `docs/design/`
- implementation changes in `src/`
- tests covering new behavior
- at least one runnable example or demo path
