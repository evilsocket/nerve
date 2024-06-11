use anyhow::Result;
use async_trait::async_trait;

pub mod ollama;

#[derive(Clone, Debug)]
pub struct ModelOptions {
    pub context_window: u32,
    pub temperature: f32,
    pub repeat_penalty: f32,
    pub top_k: u32,
}

impl Default for ModelOptions {
    fn default() -> Self {
        Self {
            context_window: 4096,
            temperature: 0.9,
            repeat_penalty: 1.3,
            top_k: 20,
        }
    }
}

#[derive(Clone, Debug)]
pub struct GeneratorOptions {
    pub system_prompt: String,
    pub prompt: String,
    pub history: Vec<Message>,
    pub model_options: ModelOptions,
}

impl GeneratorOptions {
    pub fn new(
        system_prompt: String,
        prompt: String,
        history: Vec<Message>,
        model_options: ModelOptions,
    ) -> Self {
        Self {
            system_prompt,
            prompt,
            history,
            model_options,
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
