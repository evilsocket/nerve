use std::io::{self, Write};

use clap::Parser;

use crate::agent::AgentOptions;

/// Get things done with LLMs.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(crate) struct Args {
    // TODO: refactor these in a single string -G'ollama:://localhost:11434'
    /// Generator type, currently only ollama is supported.
    #[arg(long, default_value = "ollama")]
    pub generator: String,
    /// Generator API URL.
    #[arg(long, default_value = "http://localhost")]
    pub generator_url: String,
    /// Generator API port.
    #[arg(long, default_value_t = 11434)]
    pub generator_port: u16,
    /// Generator model name.
    #[arg(short = 'M', long, default_value = "llama3")]
    pub model_name: String,
    /// Maximum number of steps to complete the task or 0 for no limit.
    #[arg(long, default_value_t = 0)]
    pub max_iterations: usize,
    /// Save the dynamic system prompt to this file if specified.
    #[arg(long)]
    pub persist_prompt_path: Option<String>,
    /// Save the dynamic state to this file if specified.
    #[arg(long)]
    pub persist_state_path: Option<String>,
    /// Tasklet file.
    #[arg(short = 'T', long)]
    pub tasklet: String,
    /// Specify the prompt if not provided by the tasklet.
    #[arg(short = 'P', long)]
    pub prompt: Option<String>,
    /// Pre define variables.
    #[arg(short = 'D', long, value_parser, num_args = 1.., value_delimiter = ' ')]
    pub define: Vec<String>,
}

impl Args {
    pub fn to_agent_options(&self) -> AgentOptions {
        AgentOptions {
            max_iterations: self.max_iterations,
            persist_prompt_path: self.persist_prompt_path.clone(),
            persist_state_path: self.persist_state_path.clone(),
        }
    }
}

pub(crate) fn get_user_input(prompt: &str) -> String {
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
