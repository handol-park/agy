# Typed Memory Model

## Goal
Replace ad-hoc string transcript state with a typed memory model that is easy to test, serialize, and replay.

This stage keeps memory intentionally small. Complexity is deferred until a later stage requires it.

## Agent Definition at Stage 003
At this stage, an agent is:
- the Stage `002` closed loop with model abstraction
- plus typed memory state updated on each `observe` step

Perception is still action-conditioned (observations come from deliberate actions), but memory is now explicit and structured.

## Why This Stage
Current state in `src/lib.rs` stores history as raw strings (`VecDeque<String>`), which:
- hides meaning (no distinction between goal seed and user reply)
- makes replay/debugging ambiguous
- limits deterministic state-based tests

Typed memory gives:
- explicit state transitions
- clearer planner inputs
- versioned serialized artifacts for debugging and regression reproduction

## Scope
- replace raw transcript strings with typed memory
- keep the runtime loop shape unchanged: `perceive -> plan -> act -> observe`
- serialize memory snapshots for replay/debugging
- add deterministic tests for memory update behavior

## Non-Goals
- long-term or cross-run memory
- retrieval indexing/vector storage
- tool-result memory normalization (covered after tool stages land)

## Proposed Core Types
Place types in `src/memory/` and re-export minimal public API from `src/lib.rs`.

```rust
pub struct Memory {
    pub goal: String,
    pub observations: Vec<ObservationRecord>,
    pub constraint_history: Vec<String>,
}

pub struct ObservationRecord {
    pub step: usize,
    pub observation: Observation,
}

pub enum Observation {
    GoalSeed { text: String },
    UserReply { text: String },
}
```

### Notes
- `goal` is first-class memory, not inferred from free-form text.
- `observations` is append-only for replay clarity.
- `constraint_history` is a derived convenience list for current planning logic.

## Memory Update Rules
Memory transitions happen only in `observe`.

1. On run start:
- initialize `Memory::goal`
- append `Observation::GoalSeed`

2. On `ActionOutcome::Observation(text)`:
- append `Observation::UserReply { text }`
- if `text.trim()` is non-empty, append to `constraint_history`

3. On `ActionOutcome::Finished(...)`:
- no new observation is appended by default

Keep rules deterministic and side-effect free.

## Planner/Runtime Access Pattern
- `perceive` reads from typed memory (`last observation`, `latest constraint`, etc.).
- `plan` consumes typed memory and emits `(thought, action)`.
- no direct indexing into raw string queues inside runtime.

Helper methods (minimal set):
- `latest_observation(&self) -> Option<&Observation>`
- `latest_constraint(&self) -> Option<&str>`

## Serialization and Replay
Use JSON with explicit schema version.

```rust
pub struct MemorySnapshot {
    pub schema_version: u32, // starts at 1
    pub step: usize,
    pub memory: Memory,
}
```

### Format
- write one snapshot per observed step
- filename example: `run-memory.jsonl`
- each line: one `MemorySnapshot` JSON object

JSONL is chosen for incremental writes and easy diffing in tests.

## Integration Plan
1. Add `src/memory/mod.rs` with `Memory`, `Observation`, and helpers.
2. Replace internal `AgentMemory` in `src/lib.rs` with `memory::Memory`.
3. Update `perceive`, `plan`, and `observe` to use typed memory accessors.
4. Add optional snapshot emission hook (in-memory in tests, file in CLI path later).
5. Keep `Action` and `ActionOutcome` unchanged in this stage.

## Test Plan
Unit tests in `src/memory/mod.rs`:
- initializes with goal seed observation
- appends user reply observation in order
- ignores empty replies for `constraint_history`
- returns correct latest observation/constraint
- round-trip serde for `MemorySnapshot`

Regression tests in `src/lib.rs`:
- existing `001/002` behavior remains unchanged with typed memory backend
- max-step behavior unchanged

## Exit Criteria
- runtime no longer stores transcript as `VecDeque<String>`
- observe-step state changes are typed and test-covered
- memory snapshots can be serialized and deserialized deterministically
- existing loop tests pass without behavioral regression

## Next Stage Link
`004` will define richer action payloads and validation. `003` should not pre-build action schemas beyond what memory requires today.
