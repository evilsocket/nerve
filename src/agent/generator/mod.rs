use std::{fmt::Display, time::Duration};

use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;
use duration_string::DurationString;
use lazy_static::lazy_static;
use regex::Regex;

use super::{rag::Embeddings, Invocation};

#[cfg(feature = "fireworks")]
mod fireworks;
#[cfg(feature = "groq")]
mod groq;
#[cfg(feature = "ollama")]
mod ollama;
#[cfg(feature = "openai")]
mod openai;

lazy_static! {
    static ref RETRY_TIME_PARSER: Regex =
        Regex::new(r"(?m)^.+try again in (.+)\. Visit.*").unwrap();
}

#[derive(Clone, Debug)]
pub struct Options {
    pub system_prompt: String,
    pub prompt: String,
    pub history: Vec<Message>,
}

impl Options {
    pub fn new(system_prompt: String, prompt: String, history: Vec<Message>) -> Self {
        Self {
            system_prompt,
            prompt,
            history,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    Agent(String, Option<Invocation>),
    Feedback(String, Option<Invocation>),
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Message::Agent(data, _) => format!("[agent]\n\n{}\n", data),
                Message::Feedback(data, _) => format!("[feedback]\n\n{}\n", data),
            }
        )
    }
}

#[async_trait]
pub trait Client: Send + Sync {
    fn new(url: &str, port: u16, model_name: &str, context_window: u32) -> Result<Self>
    where
        Self: Sized;

    async fn chat(&self, options: &Options) -> Result<String>;
    async fn embeddings(&self, text: &str) -> Result<Embeddings>;

    async fn check_rate_limit(&self, error: &str) -> bool {
        // if rate limit exceeded, parse the retry time and retry
        if let Some(caps) = RETRY_TIME_PARSER.captures_iter(error).next() {
            if caps.len() == 2 {
                let mut retry_time_str = "".to_string();

                caps.get(1)
                    .unwrap()
                    .as_str()
                    .clone_into(&mut retry_time_str);

                // DurationString can't handle decimals like Xm3.838383s
                if retry_time_str.contains('.') {
                    let (val, _) = retry_time_str.split_once('.').unwrap();
                    retry_time_str = format!("{}s", val);
                }

                if let Ok(retry_time) = retry_time_str.parse::<DurationString>() {
                    println!(
                        "{}: rate limit reached for this model, retrying in {} ...\n",
                        "WARNING".bold().yellow(),
                        retry_time,
                    );

                    tokio::time::sleep(
                        retry_time.checked_add(Duration::from_millis(1000)).unwrap(),
                    )
                    .await;

                    return true;
                } else {
                    eprintln!("can't parse '{}'", &retry_time_str);
                }
            } else {
                eprintln!("cap len wrong");
            }
        }

        return false;
    }
}

pub fn factory(
    name: &str,
    url: &str,
    port: u16,
    model_name: &str,
    context_window: u32,
) -> Result<Box<dyn Client>> {
    match name {
        #[cfg(feature = "ollama")]
        "ollama" => Ok(Box::new(ollama::OllamaClient::new(
            url,
            port,
            model_name,
            context_window,
        )?)),
        #[cfg(feature = "openai")]
        "openai" => Ok(Box::new(openai::OpenAIClient::new(
            url,
            port,
            model_name,
            context_window,
        )?)),
        #[cfg(feature = "fireworks")]
        "fireworks" => Ok(Box::new(fireworks::FireworksClient::new(
            url,
            port,
            model_name,
            context_window,
        )?)),
        #[cfg(feature = "groq")]
        "groq" => Ok(Box::new(groq::GroqClient::new(
            url,
            port,
            model_name,
            context_window,
        )?)),
        _ => Err(anyhow!("generator '{name} not supported yet")),
    }
}
