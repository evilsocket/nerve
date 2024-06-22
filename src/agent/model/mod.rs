use anyhow::Result;
use async_trait::async_trait;

mod ollama;
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
    Agent(String),
    Feedback(String),
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
        "ollama" => Ok(Box::new(ollama::OllamaClient::new(
            url,
            port,
            model_name,
            context_window,
        )?)),
        "openai" => Ok(Box::new(openai::OpenAIClient::new(
            url,
            port,
            model_name,
            context_window,
        )?)),
        _ => Err(anyhow!("generator '{name} not supported yet")),
    }
}
