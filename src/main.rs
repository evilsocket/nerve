#[macro_use]
extern crate anyhow;

use std::io::{self, Write};

use crate::agent::task::Task;
use agent::{generator, task::tasklet::Tasklet, Agent};
use clap::Parser;
use colored::Colorize;

mod agent;

// TODO: add max iterations
// TODO: add current iteration to state
// TODO: different namespaces of actions: memory, task, net?, move mouse, ui interactions, etc

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Generator type, currently only ollama is supported.
    #[arg(long, default_value = "ollama")]
    generator: String,
    /// Generator API URL.
    #[arg(long, default_value = "http://localhost")]
    generator_url: String,
    /// Generator API port.
    #[arg(long, default_value_t = 11434)]
    generator_port: u16,
    /// Generator model name.
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
    println!();
    input.trim().to_string()
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let generator = generator::factory(
        "ollama",
        &args.generator_url,
        args.generator_port,
        &args.model_name,
    )
    .expect("could not create generator");

    println!(
        "using {}@{}:{}",
        args.model_name.bold(),
        args.generator_url.dimmed(),
        args.generator_port.to_string().dimmed()
    );

    let mut tasklet: Tasklet =
        Tasklet::from_yaml_file(&args.tasklet).expect("could not read tasklet yaml file");
    if tasklet.prompt.is_none() {
        tasklet.prompt = Some(if let Some(prompt) = &args.prompt {
            prompt.to_string()
        } else {
            get_user_input("enter task> ")
        });
    }
    let task = Box::new(tasklet);

    println!(
        "{}: {}\n",
        "task".bold(),
        task.to_prompt()
            .expect("could not convert task to prompt")
            .yellow()
    );

    let mut agent = Agent::new(
        generator,
        task,
        args.persist_prompt_path,
        args.persist_state_path,
    )
    .expect("could not create agent");

    while !agent.is_state_complete() {
        if let Err(error) = agent.step().await {
            println!("ERROR: {}", &error);
        }
    }
}
