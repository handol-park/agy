use agy::model::OpenAiCompatModel;
use agy::{Action, Agent, Environment, LanguageModel, RunState, TemplateModel};
use async_trait::async_trait;
use std::env;
use std::io::{self, Write};

struct StdioEnv;

#[async_trait]
impl Environment for StdioEnv {
    async fn ask(&mut self, prompt: &str) -> String {
        read_line(prompt).unwrap_or_default()
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
    let provider = env::var("LLM_PROVIDER")
        .unwrap_or_else(|_| "openai".to_string())
        .to_lowercase();
    let (default_base_url, default_model) = match provider.as_str() {
        "glm5" | "glm-5" | "zai" | "zhipu" => (
            "https://api.z.ai/api/paas/v4".to_string(),
            "glm-5".to_string(),
        ),
        _ => (
            "https://api.openai.com/v1".to_string(),
            "gpt-4o-mini".to_string(),
        ),
    };

    let api_key = env::var("LLM_API_KEY")
        .ok()
        .or_else(|| env::var("OPENAI_API_KEY").ok())
        .or_else(|| env::var("GLM_API_KEY").ok());

    if let Some(api_key) = api_key {
        let base_url = env::var("LLM_BASE_URL")
            .ok()
            .or_else(|| env::var("OPENAI_BASE_URL").ok())
            .or_else(|| env::var("GLM_BASE_URL").ok())
            .unwrap_or(default_base_url);
        let model = env::var("LLM_MODEL")
            .ok()
            .or_else(|| env::var("OPENAI_MODEL").ok())
            .or_else(|| env::var("GLM_MODEL").ok())
            .unwrap_or(default_model);

        let llm = OpenAiCompatModel::new(base_url, api_key, model.clone());

        return (
            Box::new(llm),
            format!("openai-compatible provider={provider} model={model}"),
        );
    }

    (
        Box::new(TemplateModel),
        "template fallback (set LLM_API_KEY or OPENAI_API_KEY/GLM_API_KEY)".to_string(),
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
            Action::AskUser(payload) => {
                println!(
                    "[step {}] action: ask_user ({})",
                    trace.step, payload.prompt
                )
            }
            Action::Finish(payload) => {
                println!("[step {}] action: finish ({})", trace.step, payload.message)
            }
            Action::CallTool(payload) => {
                println!(
                    "[step {}] action: call_tool (name={} input={})",
                    trace.step, payload.tool_name, payload.input_json
                )
            }
        }
    }

    match state {
        RunState::Finished(message) => println!("\nFinal answer: {message}"),
        RunState::MaxStepsReached => println!("\nStopped after step budget was exhausted."),
    }

    Ok(())
}
