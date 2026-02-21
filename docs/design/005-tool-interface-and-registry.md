# Tool Interface and Registry

## Goal
Add a minimal, typed tool execution boundary so `Action::CallTool` can run local tools through a registry instead of returning an unsupported error.

This stage focuses on deterministic local tools and explicit error handling.

## Agent Definition at Stage 005
At this stage, an agent is:
- the Stage `004` loop with typed actions and validation
- plus a tool execution interface and registry-backed dispatch

The agent can now select and execute local tools as first-class actions.

## Why This Stage
`004` introduced `CallTool` in the action model but intentionally stubbed execution.

To make tool calls real, we need:
- a common `Tool` trait
- one registry for lookup and dispatch
- a typed tool result/error model

## Scope
- define `Tool` trait and `ToolRegistry`
- execute `Action::CallTool` via registry in runtime
- implement at least two deterministic local tools
- feed tool output into `observe` using typed memory

## Non-Goals
- remote/network tools
- dynamic plugin loading
- policy enforcement (Stage `007`)

## Proposed Interfaces
Suggested modules:
- `src/tools/mod.rs`
- `src/tools/registry.rs`
- `src/tools/builtin/calculator.rs`
- `src/tools/builtin/text_search.rs`

### Tool trait
```rust
pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn execute(&self, input_json: &serde_json::Value) -> Result<serde_json::Value, ToolError>;
}
```

### Registry
```rust
pub struct ToolRegistry {
    tools: std::collections::HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn register(&mut self, tool: Box<dyn Tool>) -> Result<(), ToolError>;
    pub fn execute(&self, name: &str, input_json: &serde_json::Value) -> Result<serde_json::Value, ToolError>;
}
```

### Errors
```rust
pub enum ToolError {
    DuplicateTool(String),
    ToolNotFound(String),
    InvalidInput(String),
    ExecutionFailed(String),
}
```

## Action Integration
In runtime `act` flow:
1. validate `Action::CallTool`
2. lookup tool by `tool_name` in registry
3. execute with `input_json`
4. return `ActionResult::ToolOutput { tool_name, output_json }`
5. convert tool failures to `ActionResult::ActionError { Runtime/Unsupported }` as appropriate

## Memory Integration
`observe` should record tool outputs as observations.

In `003`, observations include `GoalSeed` and `UserReply`.
In `005`, extend memory observation enum with:
- `ToolResult { tool_name: String, output_json: serde_json::Value }`

Keep existing behavior unchanged for user-only flows.

## Built-in Tools (minimum)
1. `calculator`
- input: `{ "expression": "1+2*3" }`
- output: `{ "result": 7.0 }`
- deterministic evaluator (no arbitrary code execution)

2. `text_search`
- input: `{ "text": "...", "query": "..." }`
- output: `{ "found": true, "count": 2 }`
- pure in-memory string matching

## Test Plan
Unit tests for tools:
- valid input -> expected output
- invalid input -> `ToolError::InvalidInput`

Unit tests for registry:
- register + execute success
- duplicate registration rejected
- unknown tool rejected

Runtime tests:
- `CallTool` action succeeds and returns `ToolOutput`
- `CallTool` unknown tool returns deterministic action error
- memory observe path records tool result observation
- existing `AskUser`/`Finish` behavior unchanged

## Exit Criteria
- runtime no longer hardcodes `CallTool` as unsupported
- tool execution is routed only through registry
- at least two deterministic local tools are available
- tool success/failure paths are covered by tests

## Next Stage Link
`006` will decouple planner logic from runtime and allow planner implementations to reason over available tools via stable interfaces.
