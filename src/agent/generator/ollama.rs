use anyhow::Result;
use async_trait::async_trait;
use ollama_rs::{
    generation::{
        chat::{request::ChatMessageRequest, ChatMessage},
        options::GenerationOptions,
    },
    Ollama,
};

use super::{Client, Message, Options};

pub struct OllamaClient {
    model: String,
    options: GenerationOptions,
    client: Ollama,
}

#[async_trait]
impl Client for OllamaClient {
    fn new(url: &str, port: u16, model_name: &str, context_window: u32) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut url = url.to_string();
        if !url.contains("://") {
            // ollama-rs is picky about this and wants the schema
            url = format!("http://{url}");
        }

        let client = Ollama::new(url.to_string(), port);
        let model = model_name.to_string();
        // Do not provide model options other than the context window size so that we'll use whatever was
        // specified in the modelfile.
        let options = GenerationOptions::default().num_ctx(context_window);

        Ok(Self {
            model,
            client,
            options,
        })
    }

    async fn chat(&self, options: &Options) -> anyhow::Result<String> {
        /*
        pub struct GenerationRequest {
            ...
            TODO: images for multimodal (see todo for screenshot action)
            pub images: Vec<Image>,
            ...
        }
        */

        // build chat history:
        //    - system prompt
        //    - user prompt
        //    - msg 0
        //    - msg 1
        //    - ...
        //    - msg n
        let mut chat_history = vec![
            ChatMessage::system(options.system_prompt.trim().to_string()),
            ChatMessage::user(options.prompt.to_string()),
        ];

        for m in &options.history {
            chat_history.push(match m {
                Message::Agent(data, _) => ChatMessage::assistant(data.trim().to_string()),
                Message::Feedback(data, _) => ChatMessage::user(data.trim().to_string()),
            });
        }

        // Do not provide model options other than the context window size so that we'll use whatever was
        // specified in the modelfile.
        let mut request = ChatMessageRequest::new(self.model.to_string(), chat_history)
            .options(self.options.clone());

        request.model_name.clone_from(&self.model);

        let res = self.client.send_chat_messages(request).await?;

        if let Some(msg) = res.message {
            Ok(msg.content)
        } else {
            log::warn!("model returned an empty message.");
            Ok("".to_string())
        }
    }
}

#[async_trait]
impl mini_rag::Embedder for OllamaClient {
    async fn embed(&self, text: &str) -> Result<mini_rag::Embeddings> {
        let resp = self
            .client
            .generate_embeddings(self.model.to_string(), text.to_string(), None)
            .await?;

        Ok(mini_rag::Embeddings::from(resp.embeddings))
    }
}
