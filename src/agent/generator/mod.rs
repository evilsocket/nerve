use std::{fmt::Display, time::Duration};

use anyhow::Result;
use async_trait::async_trait;
use duration_string::DurationString;
use history::{ChatHistory, ConversationWindow};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

use super::{namespaces::ToolOutput, state::SharedState, ToolCall};

mod anthropic;
mod deepseek;
mod fireworks;
mod google;
mod groq;
mod huggingface;
mod mistral;
mod nim;
mod novita;
mod ollama;
mod openai;
mod openai_compatible;
mod xai;

pub(crate) mod history;
mod options;

pub use options::*;

lazy_static! {
    static ref RETRY_TIME_PARSER: Regex =
        Regex::new(r"(?m)^.+try again in (.+)\. Visit.*").unwrap();
    static ref CONN_RESET_PARSER: Regex = Regex::new(r"(?m)^.+onnection reset by peer.*").unwrap();
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ChatOptions {
    pub system_prompt: Option<String>,
    pub prompt: String,
    pub history: ChatHistory,
}

impl ChatOptions {
    pub fn new(
        system_prompt: Option<String>,
        prompt: String,
        conversation: Vec<Message>,
        history_strategy: ConversationWindow,
    ) -> Self {
        let history = ChatHistory::create(conversation, history_strategy);
        Self {
            system_prompt,
            prompt,
            history,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data", rename_all = "lowercase")]
pub enum Message {
    Agent {
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_call: Option<ToolCall>,
    },
    Feedback {
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_call: Option<ToolCall>,
        result: ToolOutput,
    },
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Message::Agent {
                    content,
                    tool_call: _,
                } => {
                    format!("[agent]\n\n{}\n", content)
                }
                Message::Feedback {
                    tool_call: _,
                    result,
                } => format!("[feedback]\n\n{}\n", result),
            }
        )
    }
}

pub struct Usage {
    /// The number of input tokens which were used.
    pub input_tokens: u32,
    /// The number of output tokens which were used.
    pub output_tokens: u32,
}

pub struct ChatResponse {
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub usage: Option<Usage>,
}

pub struct SupportedFeatures {
    pub system_prompt: bool,
    pub tools: bool,
}

impl Default for SupportedFeatures {
    fn default() -> Self {
        Self {
            system_prompt: true,
            tools: false,
        }
    }
}

#[async_trait]
pub trait Client: mini_rag::Embedder + Send + Sync {
    fn new(url: &str, port: u16, model_name: &str, context_window: u32) -> Result<Self>
    where
        Self: Sized;

    async fn chat(&self, state: SharedState, options: &ChatOptions) -> Result<ChatResponse>;

    async fn check_supported_features(&self) -> Result<SupportedFeatures> {
        Ok(SupportedFeatures::default())
    }

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
                    log::warn!(
                        "rate limit reached for this model, retrying in {} ...",
                        retry_time,
                    );

                    tokio::time::sleep(
                        retry_time.checked_add(Duration::from_millis(1000)).unwrap(),
                    )
                    .await;

                    return true;
                } else {
                    log::error!("can't parse '{}'", &retry_time_str);
                }
            } else {
                log::error!("cap len wrong");
            }
        } else if CONN_RESET_PARSER.captures_iter(error).next().is_some() {
            let retry_time = Duration::from_secs(5);
            log::warn!(
                "connection reset by peer, retrying in {:?} ...",
                &retry_time,
            );

            tokio::time::sleep(retry_time).await;

            return true;
        }

        return false;
    }
}

// ugly workaround because rust doesn't support trait upcasting coercion yet

macro_rules! factory_body {
    ($name:expr, $url:expr, $port:expr, $model_name:expr, $context_window:expr) => {
        match $name {
            "ollama" => Ok(Box::new(ollama::OllamaClient::new(
                $url,
                $port,
                $model_name,
                $context_window,
            )?)),
            "openai" => Ok(Box::new(openai::OpenAIClient::new(
                $url,
                $port,
                $model_name,
                $context_window,
            )?)),
            "fireworks" => Ok(Box::new(fireworks::FireworksClient::new(
                $url,
                $port,
                $model_name,
                $context_window,
            )?)),
            "hf" => Ok(Box::new(huggingface::HuggingfaceMessageClient::new(
                $url,
                $port,
                $model_name,
                $context_window,
            )?)),
            "groq" => Ok(Box::new(groq::GroqClient::new(
                $url,
                $port,
                $model_name,
                $context_window,
            )?)),
            "novita" => Ok(Box::new(novita::NovitaClient::new(
                $url,
                $port,
                $model_name,
                $context_window,
            )?)),
            "anthropic" | "claude" => Ok(Box::new(anthropic::AnthropicClient::new(
                $url,
                $port,
                $model_name,
                $context_window,
            )?)),
            "nim" | "nvidia" => Ok(Box::new(nim::NvidiaNIMClient::new(
                $url,
                $port,
                $model_name,
                $context_window,
            )?)),
            "deepseek" => Ok(Box::new(deepseek::DeepSeekClient::new(
                $url,
                $port,
                $model_name,
                $context_window,
            )?)),
            "xai" => Ok(Box::new(xai::XAIClient::new(
                $url,
                $port,
                $model_name,
                $context_window,
            )?)),
            "mistral" => Ok(Box::new(mistral::MistralClient::new(
                $url,
                $port,
                $model_name,
                $context_window,
            )?)),
            "google" | "gemini" => Ok(Box::new(google::GoogleClient::new(
                $url,
                $port,
                $model_name,
                $context_window,
            )?)),
            "http" => Ok(Box::new(openai_compatible::OpenAiCompatibleClient::new(
                $url,
                $port,
                $model_name,
                $context_window,
            )?)),
            _ => Err(anyhow!("generator '{}' not supported yet", $name)),
        }
    };
}

pub fn factory(
    name: &str,
    url: &str,
    port: u16,
    model_name: &str,
    context_window: u32,
) -> Result<Box<dyn Client>> {
    factory_body!(name, url, port, model_name, context_window)
}

pub fn factory_embedder(
    name: &str,
    url: &str,
    port: u16,
    model_name: &str,
    context_window: u32,
) -> Result<Box<dyn mini_rag::Embedder>> {
    factory_body!(name, url, port, model_name, context_window)
}
