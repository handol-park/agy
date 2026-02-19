use std::io::{self, Write};

#[derive(Debug)]
struct Agent {
    max_steps: usize,
}

#[derive(Debug)]
enum Action {
    AskUser(String),
    Finish(String),
}

impl Agent {
    fn new(max_steps: usize) -> Self {
        Self { max_steps }
    }

    fn run(&self, goal: &str) -> io::Result<()> {
        let mut transcript: Vec<String> = Vec::new();
        let mut last_observation = format!("Goal: {goal}");

        for step in 1..=self.max_steps {
            let thought = self.plan(step, &last_observation, &transcript);
            let action = self.act(step, &thought, goal, &transcript);

            println!("\\n[step {step}] thought: {thought}");

            match action {
                Action::AskUser(prompt) => {
                    println!("[step {step}] action: ask_user");
                    let observation = read_line(&format!("[step {step}] {prompt}"))?;
                    println!("[step {step}] observation: {observation}");

                    transcript.push(format!("agent: {prompt}"));
                    transcript.push(format!("user: {observation}"));
                    last_observation = observation;
                }
                Action::Finish(message) => {
                    println!("[step {step}] action: finish");
                    println!("\\nFinal answer: {message}");
                    return Ok(());
                }
            }
        }

        println!(
            "\\nStopped after {} steps without finish action.",
            self.max_steps
        );
        Ok(())
    }

    fn plan(&self, step: usize, last_observation: &str, transcript: &[String]) -> String {
        if step == 1 {
            return "Understand the goal and gather one key requirement from the user.".to_string();
        }

        if transcript.len() >= 2 {
            return "Synthesize the user input into a concise final response.".to_string();
        }

        format!("Use latest observation: {last_observation}")
    }

    fn act(&self, step: usize, _thought: &str, goal: &str, transcript: &[String]) -> Action {
        if step == 1 {
            return Action::AskUser(format!(
                "I am working on: '{goal}'. What is the single most important constraint? "
            ));
        }

        if transcript.len() >= 2 {
            let constraint = transcript.last().map_or("none", String::as_str);
            return Action::Finish(format!(
                "Goal understood. I will prioritize this constraint: {constraint}"
            ));
        }

        Action::Finish("No additional input received. Returning minimal plan.".to_string())
    }
}

fn read_line(prompt: &str) -> io::Result<String> {
    print!("{prompt}");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn main() -> io::Result<()> {
    let goal = read_line("Enter agent goal: ")?;

    let agent = Agent::new(3);
    agent.run(&goal)
}
