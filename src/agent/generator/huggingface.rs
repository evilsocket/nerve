use anyhow::Result;
use async_trait::async_trait;

use crate::agent::{state::SharedState, Invocation};

use super::{openai::OpenAIClient, Client, Options};

pub struct HuggingfaceMessageClient {
    client: OpenAIClient,
}

#[async_trait]
impl Client for HuggingfaceMessageClient {
    fn new(url: &str, _: u16, model_name: &str, _: u32) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let message_api = format!("https://{}/v1/", url);
        let client = OpenAIClient::custom(model_name, "HF_API_TOKEN", &message_api)?;

        log::debug!("using huggingface message api @ {}", message_api);

        Ok(Self { client })
    }

    async fn chat(
        &self,
        state: SharedState,
        options: &Options,
    ) -> anyhow::Result<(String, Vec<Invocation>)> {
        self.client.chat(state, options).await
    }
}

#[async_trait]
impl mini_rag::Embedder for HuggingfaceMessageClient {
    async fn embed(&self, text: &str) -> Result<mini_rag::Embeddings> {
        self.client.embed(text).await
    }
}
