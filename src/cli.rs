use std::io::{self, Write};

use anyhow::Result;
use clap::Parser;
use lazy_static::lazy_static;
use regex::Regex;

use crate::agent::AgentOptions;

lazy_static! {
    pub static ref GENERATOR_PARSER: Regex = Regex::new(r"(?m)^(.+)://(.+)@(.+):(\d+)$").unwrap();
}

#[derive(Default)]
pub(crate) struct Generator {
    pub type_name: String,
    pub model_name: String,
    pub host: String,
    pub port: u16,
}

/// Get things done with LLMs.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(crate) struct Args {
    /// Generator string as <type>://<model name>@<host>:<port>
    #[arg(short = 'G', long, default_value = "ollama://llama3@localhost:11434")]
    pub generator: String,
    /// Tasklet file.
    #[arg(short = 'T', long)]
    pub tasklet: String,
    /// Specify the prompt if not provided by the tasklet.
    #[arg(short = 'P', long)]
    pub prompt: Option<String>,
    /// Pre define variables.
    #[arg(short = 'D', long, value_parser, num_args = 1.., value_delimiter = ' ')]
    pub define: Vec<String>,
    /// Maximum number of steps to complete the task or 0 for no limit.
    #[arg(long, default_value_t = 0)]
    pub max_iterations: usize,
    /// Save the dynamic system prompt to this file if specified.
    #[arg(long)]
    pub persist_prompt_path: Option<String>,
    /// Save the dynamic state to this file if specified.
    #[arg(long)]
    pub persist_state_path: Option<String>,
}

impl Args {
    pub fn to_agent_options(&self) -> AgentOptions {
        AgentOptions {
            max_iterations: self.max_iterations,
            persist_prompt_path: self.persist_prompt_path.clone(),
            persist_state_path: self.persist_state_path.clone(),
        }
    }

    pub fn to_generator_options(&self) -> Result<Generator> {
        let raw = self.generator.trim();
        if raw.is_empty() {
            return Err(anyhow!("generator string can't be empty".to_string()));
        }

        let mut generator = Generator::default();
        let caps = if let Some(caps) = GENERATOR_PARSER.captures_iter(raw).next() {
            caps
        } else {
            return Err(anyhow!("can't parse {raw} generator string"));
        };

        if caps.len() != 5 {
            return Err(anyhow!("can't parse {raw} generator string"));
        }

        generator.type_name = caps.get(1).unwrap().as_str().to_owned();
        generator.model_name = caps.get(2).unwrap().as_str().to_owned();
        generator.host = caps.get(3).unwrap().as_str().to_owned();
        generator.port = caps.get(4).unwrap().as_str().parse::<u16>().unwrap();

        Ok(generator)
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
