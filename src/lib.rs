use async_trait::async_trait;

pub mod memory;
pub mod scorecard;

use crate::memory::Memory;

#[derive(Debug, Clone)]
pub struct Agent {
    pub max_steps: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    AskUser(String),
    Finish(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone)]
enum ActionOutcome {
    Observation(String),
    Finished(String),
}

#[async_trait]
pub trait Environment {
    async fn ask(&mut self, prompt: &str) -> String;
}

#[async_trait]
pub trait LanguageModel {
    async fn synthesize(&self, goal: &str, constraint: &str) -> Result<String, String>;
}

#[derive(Debug, Clone, Copy)]
pub struct TemplateModel;

#[async_trait]
impl LanguageModel for TemplateModel {
    async fn synthesize(&self, goal: &str, constraint: &str) -> Result<String, String> {
        Ok(format!(
            "Plan for '{goal}': prioritize '{constraint}', keep solution minimal, and validate with one test."
        ))
    }
}

impl Agent {
    pub fn new(max_steps: usize) -> Self {
        Self { max_steps }
    }

    pub async fn run<E: Environment>(&self, goal: &str, env: &mut E) -> (RunState, Vec<StepTrace>) {
        self.run_with_model(goal, env, &TemplateModel).await
    }

    pub async fn run_with_model<E: Environment>(
        &self,
        goal: &str,
        env: &mut E,
        model: &dyn LanguageModel,
    ) -> (RunState, Vec<StepTrace>) {
        let mut memory = Memory::new(goal);
        let mut traces: Vec<StepTrace> = Vec::new();

        for step in 1..=self.max_steps {
            let perceived = self.perceive(&memory);
            let (thought, action) = self.plan(step, goal, perceived, &memory, model).await;

            traces.push(StepTrace {
                step,
                thought,
                action: action.clone(),
            });

            let outcome = self.act(&action, env).await;
            self.observe(step, &mut memory, &outcome);

            if let ActionOutcome::Finished(message) = outcome {
                return (RunState::Finished(message), traces);
            }
        }

        (RunState::MaxStepsReached, traces)
    }

    fn perceive<'a>(&self, memory: &'a Memory) -> &'a str {
        memory.latest_observation_text().unwrap_or_default()
    }

    async fn plan(
        &self,
        step: usize,
        goal: &str,
        perceived: &str,
        memory: &Memory,
        model: &dyn LanguageModel,
    ) -> (String, Action) {
        if step == 1 {
            return (
                "Understand goal and request one critical constraint.".to_string(),
                Action::AskUser(format!(
                    "I am working on '{goal}'. What single constraint matters most? "
                )),
            );
        }

        if let Some(constraint) = memory.latest_constraint() {
            let action = match model.synthesize(goal, constraint).await {
                Ok(message) => Action::Finish(message),
                Err(err) => Action::Finish(format!(
                    "Model call failed ({err}). Fallback: prioritize '{constraint}'."
                )),
            };
            return (
                "Use the language model to synthesize a concrete answer.".to_string(),
                action,
            );
        }

        (
            format!("Use latest perception '{perceived}'."),
            Action::Finish("No constraint provided. Returning minimal plan.".to_string()),
        )
    }

    async fn act<E: Environment>(&self, action: &Action, env: &mut E) -> ActionOutcome {
        match action {
            Action::AskUser(prompt) => ActionOutcome::Observation(env.ask(prompt).await),
            Action::Finish(message) => ActionOutcome::Finished(message.clone()),
        }
    }

    fn observe(&self, step: usize, memory: &mut Memory, outcome: &ActionOutcome) {
        if let ActionOutcome::Observation(observation) = outcome {
            memory.record_user_reply(step, observation.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        async fn synthesize(&self, goal: &str, constraint: &str) -> Result<String, String> {
            Ok(format!("SYNTHESIZED: {goal} | {constraint}"))
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
    async fn finishes_after_collecting_constraint_with_model_output() {
        let agent = Agent::new(3);
        let mut env = FakeEnv::new(&["low latency"]);

        let (state, traces) = agent
            .run_with_model("build an agent loop", &mut env, &FakeModelOk)
            .await;

        assert_eq!(traces.len(), 2);
        assert!(matches!(traces[0].action, Action::AskUser(_)));
        assert_eq!(
            state,
            RunState::Finished("SYNTHESIZED: build an agent loop | low latency".to_string())
        );
    }

    #[tokio::test]
    async fn falls_back_when_model_fails() {
        let agent = Agent::new(3);
        let mut env = FakeEnv::new(&["deterministic output"]);

        let (state, _traces) = agent.run_with_model("test", &mut env, &FakeModelErr).await;

        assert_eq!(
            state,
            RunState::Finished(
                "Model call failed (offline). Fallback: prioritize 'deterministic output'."
                    .to_string()
            )
        );
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
    async fn skips_synthesis_when_constraint_reply_is_empty() {
        let agent = Agent::new(3);
        let mut env = FakeEnv::new(&["   "]);

        let (state, traces) = agent
            .run_with_model("build an agent loop", &mut env, &FakeModelOk)
            .await;

        assert_eq!(traces.len(), 2);
        assert_eq!(
            state,
            RunState::Finished("No constraint provided. Returning minimal plan.".to_string())
        );
    }
}
