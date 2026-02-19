use agy::{Action, Agent, Environment, LanguageModel, RunState, TemplateModel};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::io::{self, Write};

struct StdioEnv;

#[async_trait]
impl Environment for StdioEnv {
    async fn ask(&mut self, prompt: &str) -> String {
        read_line(prompt).unwrap_or_default()
    }
}

struct OpenAiCompatModel {
    client: Client,
    base_url: String,
    api_key: String,
    model: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
}

#[derive(Serialize, Deserialize, Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChatMessage,
}

#[async_trait]
impl LanguageModel for OpenAiCompatModel {
    async fn synthesize(&self, goal: &str, constraint: &str) -> Result<String, String> {
        let system = "You are a concise agent planner. Return one short actionable answer.";
        let user =
            format!("Goal: {goal}\nConstraint: {constraint}\nReturn a minimal 2-3 sentence plan.");

        let req = ChatRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user,
                },
            ],
            temperature: 0.2,
        };

        let endpoint = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let resp = self
            .client
            .post(endpoint)
            .bearer_auth(&self.api_key)
            .json(&req)
            .send()
            .await
            .map_err(|e| format!("request error: {e}"))?
            .error_for_status()
            .map_err(|e| format!("http error: {e}"))?;

        let body: ChatResponse = resp
            .json()
            .await
            .map_err(|e| format!("decode error: {e}"))?;
        let content = body
            .choices
            .first()
            .map(|c| c.message.content.trim().to_string())
            .ok_or_else(|| "no choices in model response".to_string())?;

        if content.is_empty() {
            return Err("empty model response".to_string());
        }

        Ok(content)
    }
}

fn read_line(prompt: &str) -> io::Result<String> {
    print!("{prompt}");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn select_model() -> (Box<dyn LanguageModel>, String) {
    let api_key = env::var("OPENAI_API_KEY").ok();

    if let Some(api_key) = api_key {
        let base_url =
            env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
        let model = env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());

        let llm = OpenAiCompatModel {
            client: Client::new(),
            base_url,
            api_key,
            model: model.clone(),
        };

        return (Box::new(llm), format!("openai-compatible ({model})"));
    }

    (
        Box::new(TemplateModel),
        "template fallback (set OPENAI_API_KEY to use real LLM)".to_string(),
    )
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let goal = read_line("Enter agent goal: ")?;

    let agent = Agent::new(3);
    let mut env = StdioEnv;
    let (model, mode) = select_model();
    println!("\nModel mode: {mode}");

    let (state, traces) = agent.run_with_model(&goal, &mut env, model.as_ref()).await;

    for trace in traces {
        println!("\n[step {}] thought: {}", trace.step, trace.thought);
        match trace.action {
            Action::AskUser(prompt) => {
                println!("[step {}] action: ask_user ({prompt})", trace.step)
            }
            Action::Finish(message) => {
                println!("[step {}] action: finish ({message})", trace.step)
            }
        }
    }

    match state {
        RunState::Finished(message) => println!("\nFinal answer: {message}"),
        RunState::MaxStepsReached => println!("\nStopped after step budget was exhausted."),
    }

    Ok(())
}
