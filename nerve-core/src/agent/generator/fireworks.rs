use anyhow::Result;
use async_trait::async_trait;

use crate::agent::{state::SharedState, Invocation};

use super::{openai::OpenAIClient, ChatOptions, Client};

pub struct FireworksClient {
    client: OpenAIClient,
}

#[async_trait]
impl Client for FireworksClient {
    fn new(_: &str, _: u16, model_name: &str, _: u32) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let client = OpenAIClient::custom(
            &format!("accounts/fireworks/models/{}", model_name),
            "LLM_FIREWORKS_KEY",
            "https://api.fireworks.ai/inference/v1/",
        )?;

        Ok(Self { client })
    }

    async fn chat(
        &self,
        state: SharedState,
        options: &ChatOptions,
    ) -> anyhow::Result<(String, Vec<Invocation>)> {
        self.client.chat(state, options).await
    }
}

#[async_trait]
impl mini_rag::Embedder for FireworksClient {
    async fn embed(&self, text: &str) -> Result<mini_rag::Embeddings> {
        self.client.embed(text).await
    }
}
