use anyhow::Result;
use async_trait::async_trait;

use super::Invocation;

#[cfg(feature = "groq")]
mod groq;
#[cfg(feature = "ollama")]
mod ollama;
#[cfg(feature = "openai")]
mod openai;

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

impl Message {
    pub fn to_string(&self) -> String {
        match self {
            Message::Agent(data, _) => format!("[agent]\n\n{}\n", data),
            Message::Feedback(data, _) => format!("[feedback]\n\n{}\n", data),
        }
    }
}

#[async_trait]
pub trait Client {
    fn new(url: &str, port: u16, model_name: &str, context_window: u32) -> Result<Self>
    where
        Self: Sized;

    async fn chat(&self, options: &Options) -> Result<String>;
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
