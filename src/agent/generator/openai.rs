use anyhow::Result;
use async_trait::async_trait;
use openai_api_rust::chat::*;
use openai_api_rust::embeddings::EmbeddingsApi;
use openai_api_rust::*;

use super::{Client, Embeddings, Message, Options};

pub struct OpenAIClient {
    model: String,
    client: OpenAI,
}

#[async_trait]
impl Client for OpenAIClient {
    fn new(_: &str, _: u16, model_name: &str, _: u32) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        // Load API key from environment OPENAI_API_KEY.
        // You can also hadcode through `Auth::new(<your_api_key>)`, but it is not recommended.
        let auth = Auth::from_env().map_err(|e| anyhow!(e))?;
        let client = OpenAI::new(auth, "https://api.openai.com/v1/");
        let model = model_name.to_string();

        Ok(Self { model, client })
    }

    fn copy(&self) -> Result<Box<dyn Client>> {
        Ok(Box::new(Self {
            model: self.model.clone(),
            client: self.client.clone(),
        }))
    }

    async fn chat(&self, options: &Options) -> anyhow::Result<String> {
        let mut chat_history = vec![
            openai_api_rust::Message {
                role: Role::System,
                content: options.system_prompt.trim().to_string(),
            },
            openai_api_rust::Message {
                role: Role::User,
                content: options.prompt.trim().to_string(),
            },
        ];

        for m in &options.history {
            chat_history.push(match m {
                Message::Agent(data, _) => openai_api_rust::Message {
                    role: Role::Assistant,
                    content: data.trim().to_string(),
                },
                Message::Feedback(data, _) => openai_api_rust::Message {
                    role: Role::User,
                    content: data.trim().to_string(),
                },
            });
        }

        let body = ChatBody {
            model: self.model.to_string(),
            max_tokens: None,
            temperature: None,
            top_p: None,
            n: None,
            stream: Some(false),
            stop: None,
            presence_penalty: None,
            frequency_penalty: None,
            logit_bias: None,
            user: None,
            messages: chat_history,
        };
        let resp = self.client.chat_completion_create(&body);

        if let Err(error) = resp {
            return if self.check_rate_limit(&error.to_string()).await {
                self.chat(options).await
            } else {
                Err(anyhow!(error))
            };
        }

        let choice = resp.unwrap().choices;
        let message = &choice[0].message.as_ref().unwrap();

        Ok(message.content.to_string())
    }

    async fn embeddings(&self, text: &str) -> Result<Embeddings> {
        let body = embeddings::EmbeddingsBody {
            model: self.model.to_string(),
            input: vec![text.to_string()],
            user: None,
        };
        let resp = self.client.embeddings_create(&body);
        if let Err(error) = resp {
            return if self.check_rate_limit(&error.to_string()).await {
                self.embeddings(text).await
            } else {
                Err(anyhow!(error))
            };
        }

        let embeddings = resp.unwrap().data;
        let embedding = embeddings.as_ref().unwrap().first().unwrap();

        Ok(Embeddings::from(
            embedding.embedding.as_ref().unwrap_or(&vec![]).clone(),
        ))
    }
}
