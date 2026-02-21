# Action Model

## Goal
Define a typed, validated action space so planner output is explicit and runtime execution is predictable.

This stage keeps actions minimal. It adds only what is needed for tool execution in `005`.

## Agent Definition at Stage 004
At this stage, an agent is:
- the Stage `003` loop with typed memory
- plus a structured action model with validation before execution

An action is no longer a free-form enum payload by convention. It is a typed command with explicit input shape.

## Why This Stage
Current action handling is intentionally small (`AskUser`, `Finish`) but has no validation boundary.

Before adding tools, we need:
- one place to validate action payloads
- one execution interface that handles all action kinds uniformly
- deterministic behavior for unsupported or malformed actions

## Scope
- expand action space to include `CallTool`
- add typed payload structs per action
- validate action payloads before execution
- return structured action results to `observe`
- keep runtime loop shape unchanged (`perceive -> plan -> act -> observe`)

## Non-Goals
- tool registry implementation details (Stage `005`)
- planner architecture split (Stage `006`)
- policy enforcement (Stage `007`)

## Proposed Types
Suggested module: `src/action/mod.rs`.

```rust
pub enum Action {
    AskUser(AskUserAction),
    Finish(FinishAction),
    CallTool(CallToolAction),
}

pub struct AskUserAction {
    pub prompt: String,
}

pub struct FinishAction {
    pub message: String,
}

pub struct CallToolAction {
    pub tool_name: String,
    pub input_json: serde_json::Value,
}
```

### Validation
```rust
pub enum ActionValidationError {
    EmptyPrompt,
    EmptyFinishMessage,
    EmptyToolName,
    InvalidToolInput(String),
    UnsupportedAction(String),
}
```

Validation rules (minimal):
- `AskUser.prompt` must be non-empty after trim.
- `Finish.message` must be non-empty after trim.
- `CallTool.tool_name` must be non-empty after trim.
- `CallTool.input_json` must be JSON object for now.

## Action Execution Result
`act` should return one typed result enum regardless of action kind.

```rust
pub enum ActionResult {
    UserObservation { text: String },
    Finalized { message: String },
    ToolOutput { tool_name: String, output_json: serde_json::Value },
    ActionError { error: ActionError },
}

pub enum ActionError {
    Validation(ActionValidationError),
    Unsupported(String),
    Runtime(String),
}
```

### Runtime behavior
- validate action first
- if invalid: return `ActionResult::ActionError`
- if valid: execute
- `observe` updates typed memory from `ActionResult`

## Integration Plan
1. Add `src/action/mod.rs` with action payload types + validator.
2. Refactor existing `Action` usage in `src/lib.rs` to new typed structs.
3. Introduce `ActionResult` from `act`.
4. Keep `CallTool` path stubbed with `Unsupported(...)` until `005`.
5. Update traces to include validated action type and payload summary.

## Test Plan
Unit tests in `src/action/mod.rs`:
- valid `AskUser`/`Finish` payloads pass
- empty prompt/message/tool name fail validation
- invalid `CallTool.input_json` shape fails

Runtime tests in `src/lib.rs`:
- existing `001`/`002` behavior unchanged for `AskUser` + `Finish`
- invalid action converts to deterministic `ActionError`
- `CallTool` currently returns supported stub error until `005`

## Exit Criteria
- planner outputs typed actions with payload structs
- every action goes through validator before execution
- runtime consumes typed `ActionResult` instead of ad-hoc branching
- invalid actions are deterministic and test-covered

## Next Stage Link
`005` will implement real `Tool` trait + registry and replace the `CallTool` stub path with concrete execution.
