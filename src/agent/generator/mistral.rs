use anyhow::Result;
use async_trait::async_trait;

use crate::agent::state::SharedState;

use super::{openai::OpenAIClient, ChatOptions, ChatResponse, Client};

pub struct MistralClient {
    client: OpenAIClient,
}

#[async_trait]
impl Client for MistralClient {
    fn new(_: &str, _: u16, model_name: &str, _: u32) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let client = OpenAIClient::custom(model_name, "MISTRAL_API_KEY", "https://api.mistral.ai/v1/")?;

        Ok(Self { client })
    }

    async fn check_native_tools_support(&self) -> Result<bool> {
        self.client.check_native_tools_support().await
    }

    async fn chat(
        &self,
        state: SharedState,
        options: &ChatOptions,
    ) -> anyhow::Result<ChatResponse> {
        let response = self.client.chat(state.clone(), options).await;

        if let Err(error) = &response {
            if self.check_rate_limit(&error.to_string()).await {
                return self.chat(state, options).await;                
            }
        }

        response
    }

    async fn check_rate_limit(&self, error: &str) -> bool {
        // if message contains "Requests rate limit exceeded" return true
        if error.contains("Requests rate limit exceeded") {
            let retry_time = std::time::Duration::from_secs(5);
            log::warn!(
                "rate limit reached for this model, retrying in {:?} ...",
                &retry_time,
            );

            tokio::time::sleep(retry_time).await;

            return true;
        }

        false
    }
}

#[async_trait]
impl mini_rag::Embedder for MistralClient {
    async fn embed(&self, text: &str) -> Result<mini_rag::Embeddings> {
        self.client.embed(text).await
    }
}
