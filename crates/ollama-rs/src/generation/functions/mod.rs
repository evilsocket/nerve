pub mod pipelines;
pub mod request;
pub mod tools;

pub use crate::generation::functions::pipelines::nous_hermes::request::NousFunctionCall;
pub use crate::generation::functions::pipelines::openai::request::OpenAIFunctionCall;
pub use crate::generation::functions::request::FunctionCallRequest;
pub use tools::DDGSearcher;
pub use tools::Scraper;
pub use tools::StockScraper;

use crate::error::OllamaError;
use crate::generation::chat::request::ChatMessageRequest;
use crate::generation::chat::{ChatMessage, ChatMessageResponse};
use crate::generation::functions::pipelines::RequestParserBase;
use crate::generation::functions::tools::Tool;
use std::sync::Arc;

#[cfg(feature = "function-calling")]
impl crate::Ollama {
    fn has_system_prompt(&self, messages: &[ChatMessage], system_prompt: &str) -> bool {
        let system_message = messages.first().unwrap().clone();
        system_message.content == system_prompt
    }

    fn has_system_prompt_history(&mut self) -> bool {
        self.get_messages_history("default").is_some()
    }

    #[cfg(feature = "chat-history")]
    pub async fn send_function_call_with_history(
        &mut self,
        request: FunctionCallRequest,
        parser: Arc<dyn RequestParserBase>,
        id: String,
    ) -> Result<ChatMessageResponse, OllamaError> {
        let mut request = request;

        if !self.has_system_prompt_history() {
            let system_prompt = parser.get_system_message(&request.tools).await;
            self.set_system_response(id.clone(), system_prompt.content);

            //format input
            let formatted_query = ChatMessage::user(
                parser.format_query(&request.chat.messages.first().unwrap().content),
            );
            //replace with formatted_query with previous chat_message
            request.chat.messages.remove(0);
            request.chat.messages.insert(0, formatted_query);
        }

        let tool_call_result = self
            .send_chat_messages_with_history(
                ChatMessageRequest::new(request.chat.model_name.clone(), request.chat.messages),
                id.clone(),
            )
            .await?;

        let tool_call_content: String = tool_call_result.message.clone().unwrap().content;
        let result = parser
            .parse(
                &tool_call_content,
                request.chat.model_name.clone(),
                request.tools,
            )
            .await;

        match result {
            Ok(r) => {
                self.add_assistant_response(id.clone(), r.message.clone().unwrap().content);
                Ok(r)
            }
            Err(e) => {
                self.add_assistant_response(id.clone(), e.message.clone().unwrap().content);
                Err(OllamaError::from(e.message.unwrap().content))
            }
        }
    }

    pub async fn send_function_call(
        &self,
        request: FunctionCallRequest,
        parser: Arc<dyn RequestParserBase>,
    ) -> Result<ChatMessageResponse, OllamaError> {
        let mut request = request;

        request.chat.stream = false;
        let system_prompt = parser.get_system_message(&request.tools).await;
        let model_name = request.chat.model_name.clone();

        //Make sure the first message in chat is the system prompt
        if !self.has_system_prompt(&request.chat.messages, &system_prompt.content) {
            request.chat.messages.insert(0, system_prompt);
        }
        let result = self.send_chat_messages(request.chat).await?;
        let response_content: String = result.message.clone().unwrap().content;

        let result = parser
            .parse(&response_content, model_name, request.tools)
            .await;
        match result {
            Ok(r) => Ok(r),
            Err(e) => Err(OllamaError::from(e.message.unwrap().content)),
        }
    }
}
