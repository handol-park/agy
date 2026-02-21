use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    AskUser(AskUserAction),
    Finish(FinishAction),
    CallTool(CallToolAction),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AskUserAction {
    pub prompt: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinishAction {
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallToolAction {
    pub tool_name: String,
    pub input_json: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionValidationError {
    EmptyPrompt,
    EmptyFinishMessage,
    EmptyToolName,
    InvalidToolInput(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionError {
    Validation(ActionValidationError),
    Unsupported(String),
    Runtime(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActionResult {
    UserObservation {
        text: String,
    },
    Finalized {
        message: String,
    },
    ToolOutput {
        tool_name: String,
        output_json: Value,
    },
    ActionError {
        error: ActionError,
    },
}

impl Action {
    pub fn validate(&self) -> Result<(), ActionValidationError> {
        match self {
            Action::AskUser(payload) => {
                if payload.prompt.trim().is_empty() {
                    return Err(ActionValidationError::EmptyPrompt);
                }
                Ok(())
            }
            Action::Finish(payload) => {
                if payload.message.trim().is_empty() {
                    return Err(ActionValidationError::EmptyFinishMessage);
                }
                Ok(())
            }
            Action::CallTool(payload) => {
                if payload.tool_name.trim().is_empty() {
                    return Err(ActionValidationError::EmptyToolName);
                }
                if !payload.input_json.is_object() {
                    return Err(ActionValidationError::InvalidToolInput(
                        "tool input must be a JSON object".to_string(),
                    ));
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn validates_ask_user_payload() {
        let action = Action::AskUser(AskUserAction {
            prompt: "What matters most?".to_string(),
        });

        assert_eq!(action.validate(), Ok(()));
    }

    #[test]
    fn rejects_empty_prompt() {
        let action = Action::AskUser(AskUserAction {
            prompt: "   ".to_string(),
        });

        assert_eq!(action.validate(), Err(ActionValidationError::EmptyPrompt));
    }

    #[test]
    fn rejects_empty_finish_message() {
        let action = Action::Finish(FinishAction {
            message: "".to_string(),
        });

        assert_eq!(
            action.validate(),
            Err(ActionValidationError::EmptyFinishMessage)
        );
    }

    #[test]
    fn validates_call_tool_payload() {
        let action = Action::CallTool(CallToolAction {
            tool_name: "calculator".to_string(),
            input_json: json!({"expression": "1+1"}),
        });

        assert_eq!(action.validate(), Ok(()));
    }

    #[test]
    fn rejects_invalid_call_tool_payload() {
        let action = Action::CallTool(CallToolAction {
            tool_name: "calculator".to_string(),
            input_json: json!(["1+1"]),
        });

        assert_eq!(
            action.validate(),
            Err(ActionValidationError::InvalidToolInput(
                "tool input must be a JSON object".to_string()
            ))
        );
    }
}
