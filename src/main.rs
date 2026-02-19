use agy::{Action, Agent, Environment, RunState};
use std::io::{self, Write};

struct StdioEnv;

impl Environment for StdioEnv {
    fn ask(&mut self, prompt: &str) -> String {
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

fn main() -> io::Result<()> {
    let goal = read_line("Enter agent goal: ")?;

    let agent = Agent::new(3);
    let mut env = StdioEnv;
    let (state, traces) = agent.run(&goal, &mut env);

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
