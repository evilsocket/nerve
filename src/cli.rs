use std::io::{self, Write};

use anyhow::Result;
use clap::Parser;

use crate::agent::generator;

/// Get things done with LLMs.
#[derive(Parser, Debug, Default)]
#[command(version, about, long_about = None)]
pub(crate) struct Args {
    /// Generator string as <type>://<model name>@<host>:<port>
    #[arg(short = 'G', long, default_value = "ollama://llama3@localhost:11434")]
    pub generator: String,
    /// Embedder string as <type>://<model name>@<host>:<port>
    #[arg(
        short = 'E',
        long,
        default_value = "ollama://all-minilm@localhost:11434"
    )]
    pub embedder: String,
    /// Tasklet file.
    #[arg(short = 'T', long)]
    pub tasklet: Option<String>,
    /// Specify the prompt if not provided by the tasklet.
    #[arg(short = 'P', long)]
    pub prompt: Option<String>,
    /// Pre define variables.
    #[arg(short = 'D', long, value_parser, num_args = 1.., value_delimiter = ' ')]
    pub define: Vec<String>,
    /// Context window size.
    #[arg(long, default_value_t = 8000)]
    pub context_window: u32,
    /// Maximum number of steps to complete the task or 0 for no limit.
    #[arg(long, default_value_t = 0)]
    pub max_iterations: usize,
    /// At every step, save the current system prompt and state data to this file.
    #[arg(long)]
    pub save_to: Option<String>,
    /// Print the documentation of the available action namespaces.
    #[arg(long)]
    pub generate_doc: bool,
}

impl Args {
    pub fn to_generator_options(&self) -> Result<generator::Options> {
        generator::Options::parse(&self.generator, self.context_window)
    }

    pub fn to_embedder_options(&self) -> Result<generator::Options> {
        generator::Options::parse(&self.embedder, self.context_window)
    }
}

pub(crate) fn get_user_input(prompt: &str) -> String {
    print!("\n{}", prompt);
    let _ = io::stdout().flush();

    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_goes_into_input_above) => {}
        Err(_no_updates_is_fine) => {}
    }
    println!();
    input.trim().to_string()
}
