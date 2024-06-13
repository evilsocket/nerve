use anyhow::Result;
use async_trait::async_trait;

pub mod ollama;

// TODO: serialize this whole chat object to disk for debugging.
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
    User(String),
    Agent(String),
}

#[async_trait]
pub trait Client {
    fn new(url: &str, port: u16, model_name: &str, context_window: u32) -> Result<Self>
    where
        Self: Sized;

    async fn chat(&self, options: Options) -> Result<String>;
}

pub fn factory(
    name: &str,
    url: &str,
    port: u16,
    model_name: &str,
    context_window: u32,
) -> Result<Box<dyn Client>> {
    let gen = match name {
        "ollama" => ollama::OllamaClient::new(url, port, model_name, context_window)?,
        _ => return Err(anyhow!("generator '{name} not supported yet")),
    };

    Ok(Box::new(gen))
}
