use async_trait::async_trait;

pub mod action;
pub mod memory;
pub mod model;
pub mod planner;
pub mod scorecard;
pub mod tools;

use crate::memory::Memory;
use crate::planner::{ModelPlanner, RulePlanner};
use crate::tools::{default_registry, ToolRegistry};

pub use crate::action::{
    Action, ActionError, ActionResult, ActionValidationError, AskUserAction, CallToolAction,
    FinishAction,
};
pub use crate::planner::{PlanContext, PlanOutput, Planner};

#[derive(Debug, Clone)]
pub struct Agent {
    pub max_steps: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StepTrace {
    pub step: usize,
    pub thought: String,
    pub action: Action,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunState {
    Finished(String),
    MaxStepsReached,
}

#[async_trait]
pub trait Environment {
    async fn ask(&mut self, prompt: &str) -> String;
}

#[async_trait]
pub trait LanguageModel: Send + Sync {
    async fn synthesize(&self, goal: &str, constraint: &str) -> Result<String, String>;

    async fn complete(&self, system: &str, user: &str) -> Result<String, String> {
        self.synthesize(system, user).await
    }
}

impl Agent {
    pub fn new(max_steps: usize) -> Self {
        Self { max_steps }
    }

    pub async fn run<E: Environment>(&self, goal: &str, env: &mut E) -> (RunState, Vec<StepTrace>) {
        self.run_with_planner(goal, env, &RulePlanner).await
    }

    pub async fn run_with_model<E: Environment>(
        &self,
        goal: &str,
        env: &mut E,
        model: &dyn LanguageModel,
    ) -> (RunState, Vec<StepTrace>) {
        let planner = ModelPlanner::new(model);
        self.run_with_planner(goal, env, &planner).await
    }

    pub async fn run_with_planner<E: Environment>(
        &self,
        goal: &str,
        env: &mut E,
        planner: &dyn Planner,
    ) -> (RunState, Vec<StepTrace>) {
        let mut memory = Memory::new(goal);
        let tools = default_registry();
        let mut traces: Vec<StepTrace> = Vec::new();

        for step in 1..=self.max_steps {
            let ctx = PlanContext {
                step,
                max_steps: self.max_steps,
                memory: &memory,
                available_tools: tools.list_tools(),
            };
            let output = planner.plan_next(&ctx).await;

            traces.push(StepTrace {
                step,
                thought: output.thought,
                action: output.action.clone(),
            });

            let result = self.act(&output.action, env, &tools).await;
            self.observe(step, &mut memory, &result);

            match result {
                ActionResult::Finalized { message } => {
                    return (RunState::Finished(message), traces);
                }
                ActionResult::ActionError { error } => {
                    return (
                        RunState::Finished(format!(
                            "Action execution failed: {}",
                            self.describe_action_error(&error)
                        )),
                        traces,
                    );
                }
                ActionResult::UserObservation { .. } | ActionResult::ToolOutput { .. } => {}
            }
        }

        (RunState::MaxStepsReached, traces)
    }

    async fn act<E: Environment>(
        &self,
        action: &Action,
        env: &mut E,
        tools: &ToolRegistry,
    ) -> ActionResult {
        if let Err(err) = action.validate() {
            return ActionResult::ActionError {
                error: ActionError::Validation(err),
            };
        }

        match action {
            Action::AskUser(payload) => ActionResult::UserObservation {
                text: env.ask(&payload.prompt).await,
            },
            Action::Finish(payload) => ActionResult::Finalized {
                message: payload.message.clone(),
            },
            Action::CallTool(CallToolAction {
                tool_name,
                input_json,
            }) => match tools.execute(tool_name, input_json) {
                Ok(output_json) => ActionResult::ToolOutput {
                    tool_name: tool_name.clone(),
                    output_json,
                },
                Err(err) => ActionResult::ActionError {
                    error: ActionError::Runtime(format!("{err:?}")),
                },
            },
        }
    }

    fn observe(&self, step: usize, memory: &mut Memory, result: &ActionResult) {
        match result {
            ActionResult::UserObservation { text } => {
                memory.record_user_reply(step, text.clone());
            }
            ActionResult::ToolOutput {
                tool_name,
                output_json,
            } => {
                memory.record_tool_result(step, tool_name.clone(), output_json.clone());
            }
            ActionResult::Finalized { .. } | ActionResult::ActionError { .. } => {}
        }
    }

    fn describe_action_error(&self, error: &ActionError) -> String {
        match error {
            ActionError::Validation(err) => format!("validation error: {err:?}"),
            ActionError::Unsupported(msg) => format!("unsupported action: {msg}"),
            ActionError::Runtime(msg) => format!("runtime error: {msg}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::VecDeque;

    struct FakeEnv {
        replies: VecDeque<String>,
    }

    impl FakeEnv {
        fn new(replies: &[&str]) -> Self {
            Self {
                replies: replies.iter().map(|s| s.to_string()).collect(),
            }
        }
    }

    #[async_trait]
    impl Environment for FakeEnv {
        async fn ask(&mut self, _prompt: &str) -> String {
            self.replies.pop_front().unwrap_or_default()
        }
    }

    struct FakeModelOk;

    #[async_trait]
    impl LanguageModel for FakeModelOk {
        async fn synthesize(&self, _goal: &str, _constraint: &str) -> Result<String, String> {
            unreachable!("ModelPlanner uses complete(), not synthesize()");
        }

        async fn complete(&self, _system: &str, user: &str) -> Result<String, String> {
            // ModelPlanner includes "Step: N/M" in the user prompt.
            // Return ask_user on step 1, finish on later steps.
            if user.contains("Step: 1/") {
                Ok(r#"{"thought":"gather info","action_type":"ask_user","prompt":"What matters most?"}"#.to_string())
            } else {
                Ok(
                    r#"{"thought":"synthesized","action_type":"finish","message":"model output"}"#
                        .to_string(),
                )
            }
        }
    }

    struct FakeModelErr;

    #[async_trait]
    impl LanguageModel for FakeModelErr {
        async fn synthesize(&self, _goal: &str, _constraint: &str) -> Result<String, String> {
            Err("offline".to_string())
        }
    }

    #[tokio::test]
    async fn model_planner_asks_then_finishes() {
        let agent = Agent::new(3);
        let mut env = FakeEnv::new(&["low latency"]);

        let (state, traces) = agent
            .run_with_model("build an agent loop", &mut env, &FakeModelOk)
            .await;

        assert_eq!(traces.len(), 2);
        assert!(matches!(traces[0].action, Action::AskUser(_)));
        assert_eq!(state, RunState::Finished("model output".to_string()));
    }

    #[tokio::test]
    async fn model_planner_handles_model_error() {
        let agent = Agent::new(3);
        let mut env = FakeEnv::new(&["deterministic output"]);

        let (state, _traces) = agent.run_with_model("test", &mut env, &FakeModelErr).await;

        match state {
            RunState::Finished(msg) => assert!(msg.contains("offline")),
            other => panic!("expected Finished, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn returns_max_steps_if_budget_too_small() {
        let agent = Agent::new(1);
        let mut env = FakeEnv::new(&["anything"]);

        let (state, traces) = agent.run("test", &mut env).await;

        assert_eq!(state, RunState::MaxStepsReached);
        assert_eq!(traces.len(), 1);
    }

    #[tokio::test]
    async fn model_planner_falls_back_on_non_json_response() {
        struct PlainTextModel;

        #[async_trait]
        impl LanguageModel for PlainTextModel {
            async fn synthesize(&self, _g: &str, _c: &str) -> Result<String, String> {
                unreachable!();
            }

            async fn complete(&self, _system: &str, _user: &str) -> Result<String, String> {
                Ok("just plain text".to_string())
            }
        }

        let agent = Agent::new(3);
        let mut env = FakeEnv::new(&["constraint"]);

        let (state, traces) = agent
            .run_with_model("goal", &mut env, &PlainTextModel)
            .await;

        // Step 1: plain text → fallback Finish. Loop ends after 1 step.
        assert_eq!(traces.len(), 1);
        assert_eq!(state, RunState::Finished("just plain text".to_string()));
    }

    #[tokio::test]
    async fn returns_validation_error_for_invalid_action() {
        let agent = Agent::new(3);
        let mut env = FakeEnv::new(&["unused"]);
        let tools = default_registry();
        let invalid = Action::AskUser(AskUserAction {
            prompt: "   ".to_string(),
        });

        let result = agent.act(&invalid, &mut env, &tools).await;

        assert_eq!(
            result,
            ActionResult::ActionError {
                error: ActionError::Validation(ActionValidationError::EmptyPrompt)
            }
        );
    }

    #[tokio::test]
    async fn executes_call_tool_via_registry() {
        let agent = Agent::new(3);
        let mut env = FakeEnv::new(&["unused"]);
        let tools = default_registry();
        let action = Action::CallTool(CallToolAction {
            tool_name: "calculator".to_string(),
            input_json: json!({"expression": "1+1"}),
        });

        let result = agent.act(&action, &mut env, &tools).await;

        assert_eq!(
            result,
            ActionResult::ToolOutput {
                tool_name: "calculator".to_string(),
                output_json: json!({"result":2.0})
            }
        );
    }

    #[tokio::test]
    async fn returns_runtime_error_for_unknown_tool() {
        let agent = Agent::new(3);
        let mut env = FakeEnv::new(&["unused"]);
        let action = Action::CallTool(CallToolAction {
            tool_name: "missing".to_string(),
            input_json: json!({}),
        });

        let tools = default_registry();
        let result = agent.act(&action, &mut env, &tools).await;

        assert_eq!(
            result,
            ActionResult::ActionError {
                error: ActionError::Runtime("ToolNotFound(\"missing\")".to_string())
            }
        );
    }

    #[tokio::test]
    async fn run_with_planner_uses_custom_planner() {
        struct FakePlanner;

        #[async_trait]
        impl Planner for FakePlanner {
            async fn plan_next(&self, ctx: &PlanContext<'_>) -> PlanOutput {
                if ctx.step == 1 {
                    PlanOutput {
                        thought: "ask".to_string(),
                        action: Action::AskUser(AskUserAction {
                            prompt: "Tell me something.".to_string(),
                        }),
                    }
                } else {
                    PlanOutput {
                        thought: "done".to_string(),
                        action: Action::Finish(FinishAction {
                            message: format!("Custom plan for '{}'", ctx.memory.goal),
                        }),
                    }
                }
            }
        }

        let agent = Agent::new(3);
        let mut env = FakeEnv::new(&["user input"]);

        let (state, traces) = agent
            .run_with_planner("my goal", &mut env, &FakePlanner)
            .await;

        assert_eq!(traces.len(), 2);
        assert!(matches!(traces[0].action, Action::AskUser(_)));
        assert_eq!(
            state,
            RunState::Finished("Custom plan for 'my goal'".to_string())
        );
    }
}
