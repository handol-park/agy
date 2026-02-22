use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::LanguageModel;

pub struct OpenAiCompatModel {
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

impl OpenAiCompatModel {
    pub fn new(base_url: String, api_key: String, model: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            api_key,
            model,
        }
    }

    async fn send_chat(&self, system: &str, user: &str) -> Result<String, String> {
        let req = ChatRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user.to_string(),
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

#[async_trait]
impl LanguageModel for OpenAiCompatModel {
    async fn synthesize(&self, goal: &str, constraint: &str) -> Result<String, String> {
        let system = "You are a concise agent planner. Return one short actionable answer.";
        let user =
            format!("Goal: {goal}\nConstraint: {constraint}\nReturn a minimal 2-3 sentence plan.");
        self.send_chat(system, &user).await
    }

    async fn complete(&self, system: &str, user: &str) -> Result<String, String> {
        self.send_chat(system, user).await
    }
}
