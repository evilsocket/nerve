use anyhow::Result;
use async_trait::async_trait;

pub mod ollama;

#[derive(Clone, Debug)]
pub struct GeneratorOptions {
    pub system_prompt: String,
    pub prompt: String,
    pub history: Vec<Message>,
}

impl GeneratorOptions {
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
pub trait Generator {
    fn new(url: &str, port: u16, model_name: &str) -> Result<Self>
    where
        Self: Sized;

    async fn run(&self, options: GeneratorOptions) -> Result<String>;
}

pub fn factory(name: &str, url: &str, port: u16, model_name: &str) -> Result<Box<dyn Generator>> {
    let gen = match name {
        "ollama" => ollama::OllamaGenerator::new(url, port, model_name)?,
        _ => return Err(anyhow!("generator '{name} not supported yet")),
    };

    Ok(Box::new(gen))
}
