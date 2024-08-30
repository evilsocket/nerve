use std::io::{self, Write};

use anyhow::Result;
use clap::Parser;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    pub static ref PUBLIC_GENERATOR_PARSER: Regex = Regex::new(r"(?m)^(.+)://(.+)$").unwrap();
    pub static ref LOCAL_GENERATOR_PARSER: Regex =
        Regex::new(r"(?m)^(.+)://(.+)@([^:]+):?(\d+)?$").unwrap();
}

#[derive(Default)]
pub(crate) struct GeneratorOptions {
    pub type_name: String,
    pub model_name: String,
    pub context_window: u32,
    pub host: String,
    pub port: u16,
}

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
    fn parse_connection_string(&self, raw: &str, what: &str) -> Result<GeneratorOptions> {
        let raw = raw.trim().trim_matches(|c| c == '"' || c == '\'');
        if raw.is_empty() {
            return Err(anyhow!("{what} string can't be empty"));
        }

        let mut generator = GeneratorOptions {
            context_window: self.context_window,
            ..Default::default()
        };

        if raw.contains('@') {
            let caps = if let Some(caps) = LOCAL_GENERATOR_PARSER.captures_iter(raw).next() {
                caps
            } else {
                return Err(anyhow!("can't parse '{raw}' {what} string"));
            };

            if caps.len() != 5 {
                return Err(anyhow!(
                    "can't parse {raw} {what} string ({} captures instead of 5): {:?}",
                    caps.len(),
                    caps,
                ));
            }

            caps.get(1)
                .unwrap()
                .as_str()
                .clone_into(&mut generator.type_name);
            caps.get(2)
                .unwrap()
                .as_str()
                .clone_into(&mut generator.model_name);
            caps.get(3)
                .unwrap()
                .as_str()
                .clone_into(&mut generator.host);
            generator.port = if let Some(port) = caps.get(4) {
                port.as_str().parse::<u16>().unwrap()
            } else {
                0
            };
        } else {
            let caps = if let Some(caps) = PUBLIC_GENERATOR_PARSER.captures_iter(raw).next() {
                caps
            } else {
                return Err(anyhow!(
                    "can't parse {raw} {what} string, invalid expression"
                ));
            };

            if caps.len() != 3 {
                return Err(anyhow!(
                    "can't parse {raw} {what} string, expected 3 captures, got {}",
                    caps.len()
                ));
            }

            caps.get(1)
                .unwrap()
                .as_str()
                .clone_into(&mut generator.type_name);
            caps.get(2)
                .unwrap()
                .as_str()
                .clone_into(&mut generator.model_name);
        }

        Ok(generator)
    }

    pub fn to_generator_options(&self) -> Result<GeneratorOptions> {
        self.parse_connection_string(&self.generator, "generator")
    }

    pub fn to_embedder_options(&self) -> Result<GeneratorOptions> {
        self.parse_connection_string(&self.embedder, "embedder")
    }
}

pub(crate) fn get_user_input(prompt: &str) -> String {
    log::warn!("user prompt input required");

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

#[cfg(test)]
mod tests {
    use super::Args;

    #[test]
    fn test_wont_parse_invalid_generator() {
        let mut args = Args::default();
        args.generator = "not a valid generator".to_string();
        let ret = args.to_generator_options();
        assert!(ret.is_err());
    }

    #[test]
    fn test_parse_local_generator_full() {
        let mut args = Args::default();
        args.generator = "ollama://llama3@localhost:11434".to_string();
        let ret = args.to_generator_options().unwrap();
        assert_eq!(ret.type_name, "ollama");
        assert_eq!(ret.model_name, "llama3");
        assert_eq!(ret.host, "localhost");
        assert_eq!(ret.port, 11434);
    }

    #[test]
    fn test_parse_local_generator_without_port() {
        let mut args = Args::default();
        args.generator = "ollama://llama3@localhost".to_string();
        let ret = args.to_generator_options().unwrap();
        assert_eq!(ret.type_name, "ollama");
        assert_eq!(ret.model_name, "llama3");
        assert_eq!(ret.host, "localhost");
        assert_eq!(ret.port, 0);
    }

    #[test]
    fn test_parse_public_generator() {
        let mut args = Args::default();
        args.generator = "groq://llama3".to_string();
        let ret = args.to_generator_options().unwrap();
        assert_eq!(ret.type_name, "groq");
        assert_eq!(ret.model_name, "llama3");
        assert_eq!(ret.host, "");
        assert_eq!(ret.port, 0);
    }
}
