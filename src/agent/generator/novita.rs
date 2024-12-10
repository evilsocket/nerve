use anyhow::Result;
use async_trait::async_trait;

use crate::agent::state::SharedState;

use super::{openai::OpenAIClient, ChatOptions, ChatResponse, Client, SupportedFeatures};

pub struct NovitaClient {
    client: OpenAIClient,
}

#[async_trait]
impl Client for NovitaClient {
    fn new(_: &str, _: u16, model_name: &str, _: u32) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let client = OpenAIClient::custom(
            model_name,
            "NOVITA_API_KEY",
            "https://api.novita.ai/v3/openai/",
        )?;

        Ok(Self { client })
    }

    async fn check_supported_features(&self) -> Result<SupportedFeatures> {
        self.client.check_supported_features().await
    }

    async fn chat(
        &self,
        state: SharedState,
        options: &ChatOptions,
    ) -> anyhow::Result<ChatResponse> {
        self.client.chat(state, options).await
    }
}

#[async_trait]
impl mini_rag::Embedder for NovitaClient {
    async fn embed(&self, text: &str) -> Result<mini_rag::Embeddings> {
        self.client.embed(text).await
    }
}
