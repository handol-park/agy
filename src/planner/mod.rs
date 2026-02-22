use async_trait::async_trait;

use crate::action::{Action, AskUserAction, CallToolAction, FinishAction};
use crate::memory::Memory;
use crate::tools::ToolInfo;
use crate::LanguageModel;

#[derive(Debug, Clone)]
pub struct PlanContext<'a> {
    pub step: usize,
    pub max_steps: usize,
    pub memory: &'a Memory,
    pub available_tools: Vec<ToolInfo>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanOutput {
    pub thought: String,
    pub action: Action,
}

#[async_trait]
pub trait Planner: Send + Sync {
    async fn plan_next(&self, ctx: &PlanContext<'_>) -> PlanOutput;
}

// --- RulePlanner ---

pub struct RulePlanner;

#[async_trait]
impl Planner for RulePlanner {
    async fn plan_next(&self, ctx: &PlanContext<'_>) -> PlanOutput {
        if ctx.step == 1 {
            return PlanOutput {
                thought: "Understand goal and request one critical constraint.".to_string(),
                action: Action::AskUser(AskUserAction {
                    prompt: format!(
                        "I am working on '{}'. What single constraint matters most? ",
                        ctx.memory.goal
                    ),
                }),
            };
        }

        if let Some(constraint) = ctx.memory.latest_constraint() {
            return PlanOutput {
                thought: "Apply constraint to goal using rule-based template.".to_string(),
                action: Action::Finish(FinishAction {
                    message: format!(
                        "Plan for '{}': prioritize '{}', keep solution minimal, and validate with one test.",
                        ctx.memory.goal, constraint
                    ),
                }),
            };
        }

        PlanOutput {
            thought: "No constraint available.".to_string(),
            action: Action::Finish(FinishAction {
                message: "No constraint provided. Returning minimal plan.".to_string(),
            }),
        }
    }
}

// --- ModelPlanner ---

pub struct ModelPlanner<'m> {
    model: &'m dyn LanguageModel,
}

impl<'m> ModelPlanner<'m> {
    pub fn new(model: &'m dyn LanguageModel) -> Self {
        Self { model }
    }

    fn build_system_prompt(available_tools: &[ToolInfo]) -> String {
        let mut prompt = String::from(
            "You are an agent planner. Given the current goal, memory, and available tools, \
             decide the next action.\n\n\
             Respond with a JSON object containing:\n\
             - \"thought\": your reasoning\n\
             - \"action_type\": one of \"ask_user\", \"finish\", \"call_tool\"\n\
             - For ask_user: include \"prompt\"\n\
             - For finish: include \"message\"\n\
             - For call_tool: include \"tool_name\" and \"tool_input\" (object)\n\n\
             Respond with ONLY the JSON object, no other text.",
        );

        if !available_tools.is_empty() {
            prompt.push_str("\n\nAvailable tools:\n");
            for tool in available_tools {
                prompt.push_str(&format!("- {}: {}\n", tool.name, tool.description));
            }
        }

        prompt
    }

    fn build_user_prompt(ctx: &PlanContext<'_>) -> String {
        let mut prompt = format!(
            "Goal: {}\nStep: {}/{}\n",
            ctx.memory.goal, ctx.step, ctx.max_steps
        );

        if let Some(constraint) = ctx.memory.latest_constraint() {
            prompt.push_str(&format!("Latest constraint: {constraint}\n"));
        }

        if let Some(obs) = ctx.memory.latest_observation_text() {
            prompt.push_str(&format!("Latest observation: {obs}\n"));
        }

        prompt
    }

    fn parse_response(text: &str) -> Option<PlanOutput> {
        let parsed: serde_json::Value = serde_json::from_str(text).ok()?;
        let obj = parsed.as_object()?;

        let thought = obj.get("thought")?.as_str()?.to_string();
        let action_type = obj.get("action_type")?.as_str()?;

        let action = match action_type {
            "ask_user" => {
                let prompt = obj.get("prompt")?.as_str()?.to_string();
                Action::AskUser(AskUserAction { prompt })
            }
            "finish" => {
                let message = obj.get("message")?.as_str()?.to_string();
                Action::Finish(FinishAction { message })
            }
            "call_tool" => {
                let tool_name = obj.get("tool_name")?.as_str()?.to_string();
                let tool_input = obj.get("tool_input")?.clone();
                Action::CallTool(CallToolAction {
                    tool_name,
                    input_json: tool_input,
                })
            }
            _ => return None,
        };

        Some(PlanOutput { thought, action })
    }
}

#[async_trait]
impl Planner for ModelPlanner<'_> {
    async fn plan_next(&self, ctx: &PlanContext<'_>) -> PlanOutput {
        let system = Self::build_system_prompt(&ctx.available_tools);
        let user = Self::build_user_prompt(ctx);

        match self.model.complete(&system, &user).await {
            Ok(response) => Self::parse_response(&response).unwrap_or(PlanOutput {
                thought: "Failed to parse model response as JSON.".to_string(),
                action: Action::Finish(FinishAction { message: response }),
            }),
            Err(err) => PlanOutput {
                thought: format!("Model call failed: {err}"),
                action: Action::Finish(FinishAction {
                    message: format!("Model error: {err}"),
                }),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // --- RulePlanner tests ---

    #[tokio::test]
    async fn rule_planner_step1_asks_user() {
        let memory = Memory::new("build something");
        let ctx = PlanContext {
            step: 1,
            max_steps: 3,
            memory: &memory,
            available_tools: vec![],
        };

        let output = RulePlanner.plan_next(&ctx).await;

        assert!(matches!(output.action, Action::AskUser(_)));
        assert_eq!(
            output.thought,
            "Understand goal and request one critical constraint."
        );
    }

    #[tokio::test]
    async fn rule_planner_step2_finishes_with_constraint() {
        let mut memory = Memory::new("build something");
        memory.record_user_reply(1, "low latency".to_string());
        let ctx = PlanContext {
            step: 2,
            max_steps: 3,
            memory: &memory,
            available_tools: vec![],
        };

        let output = RulePlanner.plan_next(&ctx).await;

        match &output.action {
            Action::Finish(f) => {
                assert!(f.message.contains("low latency"));
                assert!(f.message.contains("build something"));
            }
            other => panic!("expected Finish, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn rule_planner_step2_no_constraint_finishes_minimal() {
        let mut memory = Memory::new("build something");
        memory.record_user_reply(1, "   ".to_string());
        let ctx = PlanContext {
            step: 2,
            max_steps: 3,
            memory: &memory,
            available_tools: vec![],
        };

        let output = RulePlanner.plan_next(&ctx).await;

        match &output.action {
            Action::Finish(f) => {
                assert!(f.message.contains("No constraint"));
            }
            other => panic!("expected Finish, got {other:?}"),
        }
    }

    // --- ModelPlanner tests ---

    struct FakeModelErr;

    #[async_trait]
    impl LanguageModel for FakeModelErr {
        async fn synthesize(&self, _goal: &str, _constraint: &str) -> Result<String, String> {
            Err("offline".to_string())
        }
    }

    struct FakeCompleteModel {
        response: String,
    }

    #[async_trait]
    impl LanguageModel for FakeCompleteModel {
        async fn synthesize(&self, _goal: &str, _constraint: &str) -> Result<String, String> {
            unreachable!("ModelPlanner should use complete(), not synthesize()");
        }

        async fn complete(&self, _system: &str, _user: &str) -> Result<String, String> {
            Ok(self.response.clone())
        }
    }

    #[tokio::test]
    async fn model_planner_parses_ask_user_json() {
        let model = FakeCompleteModel {
            response: r#"{"thought":"need info","action_type":"ask_user","prompt":"What next?"}"#
                .to_string(),
        };
        let planner = ModelPlanner::new(&model);
        let memory = Memory::new("test");
        let ctx = PlanContext {
            step: 1,
            max_steps: 3,
            memory: &memory,
            available_tools: vec![],
        };

        let output = planner.plan_next(&ctx).await;
        assert_eq!(output.thought, "need info");
        assert!(matches!(output.action, Action::AskUser(ref a) if a.prompt == "What next?"));
    }

    #[tokio::test]
    async fn model_planner_parses_finish_json() {
        let model = FakeCompleteModel {
            response: r#"{"thought":"done","action_type":"finish","message":"All done."}"#
                .to_string(),
        };
        let planner = ModelPlanner::new(&model);
        let memory = Memory::new("test");
        let ctx = PlanContext {
            step: 2,
            max_steps: 3,
            memory: &memory,
            available_tools: vec![],
        };

        let output = planner.plan_next(&ctx).await;
        assert_eq!(output.thought, "done");
        assert!(matches!(output.action, Action::Finish(ref f) if f.message == "All done."));
    }

    #[tokio::test]
    async fn model_planner_parses_call_tool_json() {
        let model = FakeCompleteModel {
            response: r#"{"thought":"calculate","action_type":"call_tool","tool_name":"calculator","tool_input":{"expression":"1+1"}}"#.to_string(),
        };
        let planner = ModelPlanner::new(&model);
        let memory = Memory::new("test");
        let ctx = PlanContext {
            step: 2,
            max_steps: 3,
            memory: &memory,
            available_tools: vec![ToolInfo {
                name: "calculator".to_string(),
                description: "math".to_string(),
            }],
        };

        let output = planner.plan_next(&ctx).await;
        assert_eq!(output.thought, "calculate");
        match &output.action {
            Action::CallTool(ct) => {
                assert_eq!(ct.tool_name, "calculator");
                assert_eq!(ct.input_json, json!({"expression": "1+1"}));
            }
            other => panic!("expected CallTool, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn model_planner_falls_back_on_invalid_json() {
        let model = FakeCompleteModel {
            response: "not json at all".to_string(),
        };
        let planner = ModelPlanner::new(&model);
        let memory = Memory::new("test");
        let ctx = PlanContext {
            step: 1,
            max_steps: 3,
            memory: &memory,
            available_tools: vec![],
        };

        let output = planner.plan_next(&ctx).await;
        assert!(output.thought.contains("Failed to parse"));
        match &output.action {
            Action::Finish(f) => assert_eq!(f.message, "not json at all"),
            other => panic!("expected Finish fallback, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn model_planner_handles_model_error() {
        let model = FakeModelErr;
        let planner = ModelPlanner::new(&model);
        let memory = Memory::new("test");
        let ctx = PlanContext {
            step: 1,
            max_steps: 3,
            memory: &memory,
            available_tools: vec![],
        };

        let output = planner.plan_next(&ctx).await;
        assert!(output.thought.contains("Model call failed"));
        match &output.action {
            Action::Finish(f) => assert!(f.message.contains("offline")),
            other => panic!("expected Finish error, got {other:?}"),
        }
    }

    // --- parse_response unit tests ---

    #[test]
    fn parse_response_returns_none_for_unknown_action_type() {
        let result = ModelPlanner::parse_response(
            r#"{"thought":"x","action_type":"unknown","message":"y"}"#,
        );
        assert!(result.is_none());
    }

    #[test]
    fn parse_response_returns_none_for_missing_fields() {
        let result = ModelPlanner::parse_response(r#"{"thought":"x","action_type":"finish"}"#);
        assert!(result.is_none());
    }
}
