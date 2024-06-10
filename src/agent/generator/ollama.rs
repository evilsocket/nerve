use async_trait::async_trait;
use ollama_rs::{
    generation::{completion::request::GenerationRequest, options::GenerationOptions},
    Ollama,
};

use super::Generator;

pub struct OllamaGenerator {
    model: String,
    client: Ollama,
}

#[async_trait]
impl Generator for OllamaGenerator {
    fn new(url: &str, port: u16, model_name: &str) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let client = Ollama::new(url.to_string(), port);
        let model = model_name.to_string();

        Ok(Self { model, client })
    }

    async fn run(&self, system_prompt: &str, prompt: &str) -> anyhow::Result<String> {
        /*
        pub struct GenerationRequest {
            ...
            TODO: images for multimodal
            pub images: Vec<Image>,
            ...
        }
        */

        let req = GenerationRequest::new(self.model.to_owned(), prompt.to_owned())
            .system(system_prompt.to_owned())
            // TODO: allow user to specify these options
            .options(
                GenerationOptions::default()
                    .num_ctx(10000)
                    .temperature(0.9)
                    .repeat_penalty(1.3)
                    .top_k(20),
            );
        let res = self.client.generate(req).await?;
        Ok(res.response)
    }
}
