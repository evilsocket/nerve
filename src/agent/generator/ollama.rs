use async_trait::async_trait;

use ollama_rs::{
    generation::{
        chat::{request::ChatMessageRequest, ChatMessage},
        options::GenerationOptions,
    },
    Ollama,
};

use super::{Generator, Message};

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

    async fn run(
        &self,
        system_prompt: &str,
        prompt: &str,
        history: Vec<Message>,
    ) -> anyhow::Result<String> {
        /*
        pub struct GenerationRequest {
            ...
            TODO: images for multimodal (see todo for screenshot action)
            pub images: Vec<Image>,
            ...
        }
        */

        // TODO: allow user to specify these options
        let options = GenerationOptions::default()
            .num_ctx(10000)
            .temperature(0.9)
            .repeat_penalty(1.3)
            .top_k(20);

        // build chat history:
        //    - system prompt
        //    - user prompt
        //    - msg 0
        //    - msg 1
        //    - ...
        //    - msg n
        let mut chat_history = vec![
            ChatMessage::system(system_prompt.trim().to_string()),
            ChatMessage::user(prompt.to_string()),
        ];

        for m in history {
            chat_history.push(match m {
                Message::Agent(data) => ChatMessage::assistant(data.trim().to_string()),
                Message::User(data) => ChatMessage::user(data.trim().to_string()),
            });
        }
        // chat_history.push(ChatMessage::user(prompt.to_string()));

        /*
        println!("------------------------------------------------");
        println!("[CHAT]");
        use colored::Colorize;
        for msg in &chat_history {
            if msg.role == ollama_rs::generation::chat::MessageRole::System {
                println!("{}", "[system prompt]".yellow());
            } else if msg.role == ollama_rs::generation::chat::MessageRole::Assistant {
                println!("[{}] {}", "agent".bold(), &msg.content);
            } else {
                println!("  {}", msg.content.trim());
            }
        }
        println!("------------------------------------------------");
        println!("");
         */

        let mut request =
            ChatMessageRequest::new(self.model.to_string(), chat_history).options(options);

        request.model_name.clone_from(&self.model);

        let res = self.client.send_chat_messages(request).await?;

        if let Some(msg) = res.message {
            Ok(msg.content)
        } else {
            println!("WARNING: model returned an empty message.");
            Ok("".to_string())
        }
    }
}
