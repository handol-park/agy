use std::collections::VecDeque;

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

pub trait Environment {
    fn ask(&mut self, prompt: &str) -> String;
}

impl Agent {
    pub fn new(max_steps: usize) -> Self {
        Self { max_steps }
    }

    pub fn run<E: Environment>(&self, goal: &str, env: &mut E) -> (RunState, Vec<StepTrace>) {
        let mut transcript: VecDeque<String> = VecDeque::new();
        let mut last_observation = format!("Goal: {goal}");
        let mut traces: Vec<StepTrace> = Vec::new();

        for step in 1..=self.max_steps {
            let thought = self.plan(step, &last_observation, &transcript);
            let action = self.act(step, goal, &transcript);

            traces.push(StepTrace {
                step,
                thought,
                action: action.clone(),
            });

            match action {
                Action::AskUser(prompt) => {
                    let observation = env.ask(&prompt);
                    transcript.push_back(observation.clone());
                    last_observation = observation;
                }
                Action::Finish(message) => return (RunState::Finished(message), traces),
            }
        }

        (RunState::MaxStepsReached, traces)
    }

    fn plan(&self, step: usize, last_observation: &str, transcript: &VecDeque<String>) -> String {
        if step == 1 {
            return "Understand goal and request one critical constraint.".to_string();
        }

        if !transcript.is_empty() {
            return "Synthesize constraint and produce final answer.".to_string();
        }

        format!("Use latest observation: {last_observation}")
    }

    fn act(&self, step: usize, goal: &str, transcript: &VecDeque<String>) -> Action {
        if step == 1 {
            return Action::AskUser(format!(
                "I am working on '{goal}'. What single constraint matters most? "
            ));
        }

        if let Some(constraint) = transcript.back() {
            return Action::Finish(format!(
                "Goal understood. I will optimize for: {constraint}"
            ));
        }

        Action::Finish("No constraint provided. Returning minimal plan.".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    impl Environment for FakeEnv {
        fn ask(&mut self, _prompt: &str) -> String {
            self.replies.pop_front().unwrap_or_default()
        }
    }

    #[test]
    fn finishes_after_collecting_constraint() {
        let agent = Agent::new(3);
        let mut env = FakeEnv::new(&["low latency"]);

        let (state, traces) = agent.run("build an agent loop", &mut env);

        assert_eq!(traces.len(), 2);
        assert!(matches!(traces[0].action, Action::AskUser(_)));
        assert_eq!(
            state,
            RunState::Finished("Goal understood. I will optimize for: low latency".to_string())
        );
    }

    #[test]
    fn returns_max_steps_if_budget_too_small() {
        let agent = Agent::new(1);
        let mut env = FakeEnv::new(&["anything"]);

        let (state, traces) = agent.run("test", &mut env);

        assert_eq!(state, RunState::MaxStepsReached);
        assert_eq!(traces.len(), 1);
    }
}
