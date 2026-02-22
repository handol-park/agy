use serde_json::{json, Value};
use std::collections::HashMap;

pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn execute(&self, input_json: &Value) -> Result<Value, ToolError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolError {
    DuplicateTool(String),
    ToolNotFound(String),
    InvalidInput(String),
    ExecutionFailed(String),
}

#[derive(Default)]
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, tool: Box<dyn Tool>) -> Result<(), ToolError> {
        let name = tool.name().to_string();
        if self.tools.contains_key(&name) {
            return Err(ToolError::DuplicateTool(name));
        }
        self.tools.insert(name, tool);
        Ok(())
    }

    pub fn execute(&self, name: &str, input_json: &Value) -> Result<Value, ToolError> {
        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| ToolError::ToolNotFound(name.to_string()))?;
        tool.execute(input_json)
    }

    pub fn list_tools(&self) -> Vec<ToolInfo> {
        let mut tools: Vec<ToolInfo> = self
            .tools
            .values()
            .map(|t| ToolInfo {
                name: t.name().to_string(),
                description: t.description().to_string(),
            })
            .collect();
        tools.sort_by(|a, b| a.name.cmp(&b.name));
        tools
    }
}

pub fn default_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    registry
        .register(Box::new(CalculatorTool))
        .expect("calculator should register");
    registry
        .register(Box::new(TextSearchTool))
        .expect("text_search should register");
    registry
}

struct CalculatorTool;

impl Tool for CalculatorTool {
    fn name(&self) -> &'static str {
        "calculator"
    }

    fn description(&self) -> &'static str {
        "Evaluate a basic arithmetic expression"
    }

    fn execute(&self, input_json: &Value) -> Result<Value, ToolError> {
        let expression = input_json
            .get("expression")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ToolError::InvalidInput("expected string field 'expression'".to_string())
            })?;

        let value = evaluate_expression(expression)?;
        Ok(json!({ "result": value }))
    }
}

struct TextSearchTool;

impl Tool for TextSearchTool {
    fn name(&self) -> &'static str {
        "text_search"
    }

    fn description(&self) -> &'static str {
        "Count substring matches inside a text"
    }

    fn execute(&self, input_json: &Value) -> Result<Value, ToolError> {
        let text = input_json
            .get("text")
            .and_then(Value::as_str)
            .ok_or_else(|| ToolError::InvalidInput("expected string field 'text'".to_string()))?;
        let query = input_json
            .get("query")
            .and_then(Value::as_str)
            .ok_or_else(|| ToolError::InvalidInput("expected string field 'query'".to_string()))?;

        if query.is_empty() {
            return Err(ToolError::InvalidInput(
                "query must not be empty".to_string(),
            ));
        }

        let count = text.match_indices(query).count();
        Ok(json!({
            "found": count > 0,
            "count": count,
        }))
    }
}

fn evaluate_expression(expression: &str) -> Result<f64, ToolError> {
    let mut parser = ExpressionParser::new(expression);
    let value = parser.parse_expression()?;
    parser.skip_whitespace();
    if !parser.is_eof() {
        return Err(ToolError::InvalidInput(
            "unexpected trailing input".to_string(),
        ));
    }
    Ok(value)
}

struct ExpressionParser<'a> {
    src: &'a [u8],
    pos: usize,
}

impl<'a> ExpressionParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            src: input.as_bytes(),
            pos: 0,
        }
    }

    fn parse_expression(&mut self) -> Result<f64, ToolError> {
        let mut value = self.parse_term()?;
        loop {
            self.skip_whitespace();
            if self.consume_char('+') {
                value += self.parse_term()?;
            } else if self.consume_char('-') {
                value -= self.parse_term()?;
            } else {
                break;
            }
        }
        Ok(value)
    }

    fn parse_term(&mut self) -> Result<f64, ToolError> {
        let mut value = self.parse_factor()?;
        loop {
            self.skip_whitespace();
            if self.consume_char('*') {
                value *= self.parse_factor()?;
            } else if self.consume_char('/') {
                let divisor = self.parse_factor()?;
                if divisor == 0.0 {
                    return Err(ToolError::ExecutionFailed("division by zero".to_string()));
                }
                value /= divisor;
            } else {
                break;
            }
        }
        Ok(value)
    }

    fn parse_factor(&mut self) -> Result<f64, ToolError> {
        self.skip_whitespace();
        if self.consume_char('(') {
            let value = self.parse_expression()?;
            self.skip_whitespace();
            if !self.consume_char(')') {
                return Err(ToolError::InvalidInput("missing ')'".to_string()));
            }
            return Ok(value);
        }
        self.parse_number()
    }

    fn parse_number(&mut self) -> Result<f64, ToolError> {
        self.skip_whitespace();
        let start = self.pos;
        while self.pos < self.src.len()
            && (self.src[self.pos].is_ascii_digit() || self.src[self.pos] == b'.')
        {
            self.pos += 1;
        }

        if self.pos == start {
            return Err(ToolError::InvalidInput("expected number".to_string()));
        }

        let s = std::str::from_utf8(&self.src[start..self.pos])
            .map_err(|_| ToolError::InvalidInput("invalid utf8 in expression".to_string()))?;
        s.parse::<f64>()
            .map_err(|_| ToolError::InvalidInput("invalid number".to_string()))
    }

    fn consume_char(&mut self, c: char) -> bool {
        if self.pos < self.src.len() && self.src[self.pos] == c as u8 {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.src.len() && self.src[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.src.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_executes_registered_tool() {
        let registry = default_registry();
        let out = registry
            .execute("calculator", &json!({"expression":"1+2*3"}))
            .expect("tool should execute");
        assert_eq!(out, json!({"result": 7.0}));
    }

    #[test]
    fn registry_rejects_unknown_tool() {
        let registry = default_registry();
        let err = registry
            .execute("missing", &json!({}))
            .expect_err("unknown tool should fail");
        assert_eq!(err, ToolError::ToolNotFound("missing".to_string()));
    }

    #[test]
    fn text_search_counts_matches() {
        let registry = default_registry();
        let out = registry
            .execute("text_search", &json!({"text":"abaaba", "query":"aba"}))
            .expect("tool should execute");
        assert_eq!(out, json!({"found": true, "count": 2}));
    }

    #[test]
    fn calculator_rejects_invalid_input() {
        let registry = default_registry();
        let err = registry
            .execute("calculator", &json!({"expression":"1+"}))
            .expect_err("invalid expression should fail");
        assert_eq!(err, ToolError::InvalidInput("expected number".to_string()));
    }

    #[test]
    fn list_tools_returns_registered_tools() {
        let registry = default_registry();
        let tools = registry.list_tools();
        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0].name, "calculator");
        assert_eq!(
            tools[0].description,
            "Evaluate a basic arithmetic expression"
        );
        assert_eq!(tools[1].name, "text_search");
        assert_eq!(
            tools[1].description,
            "Count substring matches inside a text"
        );
    }

    #[test]
    fn list_tools_empty_for_empty_registry() {
        let registry = ToolRegistry::new();
        let tools = registry.list_tools();
        assert!(tools.is_empty());
    }
}
