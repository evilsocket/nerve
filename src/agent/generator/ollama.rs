use async_trait::async_trait;

use ollama_rs::{
    generation::chat::{request::ChatMessageRequest, ChatMessage},
    Ollama,
};

use super::{Generator, GeneratorOptions, Message};

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
        let mut url = url.to_string();
        if !url.contains("://") {
            // ollama-rs is picky about this and wants the schema
            url = format!("http://{url}");
        }

        let client = Ollama::new(url.to_string(), port);
        let model = model_name.to_string();

        Ok(Self { model, client })
    }

    async fn run(&self, options: GeneratorOptions) -> anyhow::Result<String> {
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

        for m in options.history {
            chat_history.push(match m {
                Message::Agent(data) => ChatMessage::assistant(data.trim().to_string()),
                Message::User(data) => ChatMessage::user(data.trim().to_string()),
            });
        }

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

        // Do not provide model options so that we'll use whatever was specified in the modelfile.
        let mut request = ChatMessageRequest::new(self.model.to_string(), chat_history);

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
