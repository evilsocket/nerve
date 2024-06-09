#[macro_use]
extern crate anyhow;

use std::io::{self, Write};

use crate::agent::task::Task;
use agent::{task::tasklet::Tasklet, Agent};
use clap::Parser;
use colored::Colorize;
use ollama_rs::Ollama;

mod agent;
mod example_tasks;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Ollama API URL.
    #[arg(long, default_value = "http://localhost")]
    ollama_url: String,
    /// Ollama API port.
    #[arg(long, default_value_t = 11434)]
    ollama_port: u16,
    /// Model name.
    #[arg(long, default_value = "llama3")]
    model_name: String,
    /// Save the dynamic system prompt to this file if specified.
    #[arg(long)]
    persist_prompt_path: Option<String>,
    /// Save the dynamic state to this file if specified.
    #[arg(long)]
    persist_state_path: Option<String>,
    /// Tasklet file.
    #[arg(short, long)]
    tasklet: String,
    /// Specify the prompt if not provided by the tasklet.
    #[arg(short, long)]
    prompt: Option<String>,
}

pub fn get_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    let _ = io::stdout().flush();

    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_goes_into_input_above) => {}
        Err(_no_updates_is_fine) => {}
    }
    input.trim().to_string()
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let mut tasklet: Tasklet = Tasklet::from_yaml_file(&args.tasklet).unwrap();
    if tasklet.prompt.is_none() {
        tasklet.prompt = Some(if let Some(prompt) = &args.prompt {
            prompt.to_string()
        } else {
            get_user_input("enter task> ")
        });
    }

    let task = Box::new(tasklet);

    println!(
        "{}: {}:{}",
        "server".bold(),
        &args.ollama_url,
        args.ollama_port
    );
    println!("{}: {}", "model".bold(), args.model_name);
    println!(
        "{}: {}\n",
        "task".bold(),
        task.to_prompt().unwrap().yellow()
    );

    let ollama = Ollama::new(args.ollama_url.to_string(), args.ollama_port);
    let mut agent = Agent::new(
        ollama,
        args.model_name.to_string(),
        task,
        args.persist_prompt_path,
        args.persist_state_path,
    )
    .unwrap();

    while !agent.is_state_complete() {
        if let Err(error) = agent.step().await {
            println!("ERROR: {}", &error);
        }
    }
}
