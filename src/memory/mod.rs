use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const MEMORY_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Memory {
    pub goal: String,
    pub observations: Vec<ObservationRecord>,
    pub constraint_history: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservationRecord {
    pub step: usize,
    pub observation: Observation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Observation {
    GoalSeed {
        text: String,
    },
    UserReply {
        text: String,
    },
    ToolResult {
        tool_name: String,
        output_json: Value,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemorySnapshot {
    pub schema_version: u32,
    pub step: usize,
    pub memory: Memory,
}

impl Memory {
    pub fn new(goal: &str) -> Self {
        Self {
            goal: goal.to_string(),
            observations: vec![ObservationRecord {
                step: 0,
                observation: Observation::GoalSeed {
                    text: format!("Goal: {goal}"),
                },
            }],
            constraint_history: Vec::new(),
        }
    }

    pub fn record_user_reply(&mut self, step: usize, text: String) {
        if !text.trim().is_empty() {
            self.constraint_history.push(text.clone());
        }

        self.observations.push(ObservationRecord {
            step,
            observation: Observation::UserReply { text },
        });
    }

    pub fn latest_observation(&self) -> Option<&Observation> {
        self.observations.last().map(|r| &r.observation)
    }

    pub fn latest_observation_text(&self) -> Option<&str> {
        match self.latest_observation() {
            Some(Observation::GoalSeed { text }) => Some(text.as_str()),
            Some(Observation::UserReply { text }) => Some(text.as_str()),
            Some(Observation::ToolResult { .. }) => Some("[tool_result]"),
            None => None,
        }
    }

    pub fn latest_constraint(&self) -> Option<&str> {
        self.constraint_history.last().map(String::as_str)
    }

    pub fn snapshot(&self, step: usize) -> MemorySnapshot {
        MemorySnapshot {
            schema_version: MEMORY_SCHEMA_VERSION,
            step,
            memory: self.clone(),
        }
    }

    pub fn record_tool_result(&mut self, step: usize, tool_name: String, output_json: Value) {
        self.observations.push(ObservationRecord {
            step,
            observation: Observation::ToolResult {
                tool_name,
                output_json,
            },
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initializes_with_goal_seed_observation() {
        let memory = Memory::new("build an agent");

        assert_eq!(memory.goal, "build an agent");
        assert_eq!(memory.observations.len(), 1);
        assert_eq!(
            memory.observations[0],
            ObservationRecord {
                step: 0,
                observation: Observation::GoalSeed {
                    text: "Goal: build an agent".to_string()
                }
            }
        );
    }

    #[test]
    fn appends_user_reply_observations_in_order() {
        let mut memory = Memory::new("x");
        memory.record_user_reply(1, "first".to_string());
        memory.record_user_reply(2, "second".to_string());

        assert_eq!(memory.observations.len(), 3);
        assert_eq!(memory.observations[1].step, 1);
        assert_eq!(memory.observations[2].step, 2);
    }

    #[test]
    fn ignores_empty_reply_for_constraint_history() {
        let mut memory = Memory::new("x");
        memory.record_user_reply(1, "   ".to_string());
        memory.record_user_reply(2, "latency".to_string());

        assert_eq!(memory.constraint_history, vec!["latency".to_string()]);
    }

    #[test]
    fn returns_latest_observation_and_constraint() {
        let mut memory = Memory::new("x");
        memory.record_user_reply(1, "determinism".to_string());

        assert_eq!(
            memory.latest_observation(),
            Some(&Observation::UserReply {
                text: "determinism".to_string()
            })
        );
        assert_eq!(memory.latest_constraint(), Some("determinism"));
        assert_eq!(memory.latest_observation_text(), Some("determinism"));
    }

    #[test]
    fn snapshot_round_trips_through_json() {
        let mut memory = Memory::new("x");
        memory.record_user_reply(1, "determinism".to_string());
        let snapshot = memory.snapshot(1);

        let encoded = serde_json::to_string(&snapshot).expect("snapshot should serialize");
        let decoded: MemorySnapshot =
            serde_json::from_str(&encoded).expect("snapshot should deserialize");

        assert_eq!(decoded, snapshot);
        assert_eq!(decoded.schema_version, MEMORY_SCHEMA_VERSION);
    }

    #[test]
    fn records_tool_result_observation() {
        let mut memory = Memory::new("x");
        memory.record_tool_result(
            1,
            "calculator".to_string(),
            serde_json::json!({"result":2.0}),
        );

        assert_eq!(
            memory.latest_observation(),
            Some(&Observation::ToolResult {
                tool_name: "calculator".to_string(),
                output_json: serde_json::json!({"result":2.0})
            })
        );
    }
}
