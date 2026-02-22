# Planner Architecture

## Goal
Extract decision logic from the `Agent` runtime into a pluggable `Planner` trait, allowing rule-based and model-driven planning strategies to be swapped without modifying loop mechanics.

## Agent Definition at Stage 006
At this stage, an agent is:
- the Stage `005` loop with typed actions, validation, and tool execution
- plus a decoupled planner module that produces actions from typed context

The runtime orchestrates only loop mechanics; all decision logic lives in planner implementations.

## Why This Stage
The `plan()` method in `Agent` hardcodes step-1/step-2 logic directly in the runtime. It cannot produce `CallTool` actions even though the tool registry is wired in. To support different planning strategies (deterministic rules, model-driven, hybrid), decision logic must be extracted behind a trait boundary.

## Scope
- define `Planner` trait with `plan_next()` method
- create `PlanContext` (typed input) and `PlanOutput` (thought + action)
- implement two planners: `RulePlanner` (deterministic), `ModelPlanner` (model-driven)
- add `ToolInfo` and `list_tools()` to `ToolRegistry` for read-only introspection
- add `complete(system, user)` default method to `LanguageModel` trait
- refactor `Agent` runtime to delegate to planners via `run_with_planner()`

## Non-Goals
- prompt templates or policy enforcement (Stage `007`)
- multi-step planning or plan revision
- async tool execution
- planner composition or chaining

## Proposed Interfaces

### PlanContext and PlanOutput
```rust
pub struct PlanContext<'a> {
    pub step: usize,
    pub max_steps: usize,
    pub memory: &'a Memory,
    pub available_tools: Vec<ToolInfo>,
}

pub struct PlanOutput {
    pub thought: String,
    pub action: Action,
}
```

### Planner trait
```rust
#[async_trait]
pub trait Planner: Send + Sync {
    async fn plan_next(&self, ctx: &PlanContext<'_>) -> PlanOutput;
}
```

### ToolInfo
```rust
#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
}
```

### LanguageModel extension
```rust
async fn complete(&self, system: &str, user: &str) -> Result<String, String> {
    self.synthesize(system, user).await
}
```

## Planner Implementations

### RulePlanner
Deterministic, no model dependency. Step 1: `AskUser`. Step 2+: `Finish` with template using goal and constraint. Used by `run()` when no model is configured.

### ModelPlanner
Wraps `&dyn LanguageModel`. Sends structured prompt via `model.complete()`, parses JSON response into any `Action` type (including `CallTool`). Falls back to `Action::Finish` with raw text on parse failure. Used by `run_with_model()`.

## Runtime Integration
- `run_with_planner(&self, goal, env, planner)` -- new primary loop method
- `run_with_model()` wraps model in `ModelPlanner`, delegates to `run_with_planner`
- `run()` uses `RulePlanner`, delegates to `run_with_planner`
- `plan()`, `perceive()`, and `TemplateModel` removed from `Agent`

## Test Plan
Unit tests for planners:
- `RulePlanner` step 1 returns `AskUser`, step 2+ returns `Finish`
- `ModelPlanner` parses valid JSON into correct `Action` types
- `ModelPlanner` falls back on invalid JSON
- `ModelPlanner` handles model errors gracefully

Unit tests for ToolInfo:
- `list_tools` returns registered tools
- `list_tools` empty for empty registry

Integration tests:
- `run_with_planner` with custom `FakePlanner`

## Exit Criteria
- runtime orchestrates only loop mechanics and execution
- planner logic is unit-testable without environment side effects
- `cargo clippy` clean

## Next Stage Link
Stage `007` will add prompt templates and policy constraints, building on the planner architecture to inject guardrails at the planning stage.
