use std::collections::HashMap;

#[cfg(all(feature = "chat-history", feature = "stream"))]
use async_stream::stream;
use serde::{Deserialize, Serialize};

use crate::Ollama;
pub mod request;
use super::images::Image;
use request::ChatMessageRequest;

#[cfg(feature = "chat-history")]
use crate::history::MessagesHistory;

#[cfg(feature = "stream")]
/// A stream of `ChatMessageResponse` objects
pub type ChatMessageResponseStream =
    std::pin::Pin<Box<dyn tokio_stream::Stream<Item = Result<ChatMessageResponse, ()>> + Send>>;

impl Ollama {
    #[cfg(feature = "stream")]
    /// Chat message generation with streaming.
    /// Returns a stream of `ChatMessageResponse` objects
    pub async fn send_chat_messages_stream(
        &self,
        request: ChatMessageRequest,
    ) -> crate::error::Result<ChatMessageResponseStream> {
        use tokio_stream::StreamExt;

        let mut request = request;
        request.stream = true;

        let url = format!("{}api/chat", self.url_str());
        let serialized = serde_json::to_string(&request)
            .map_err(|e| e.to_string())
            .unwrap();
        let res = self
            .reqwest_client
            .post(url)
            .body(serialized)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            return Err(res.text().await.unwrap_or_else(|e| e.to_string()).into());
        }

        let stream = Box::new(res.bytes_stream().map(|res| match res {
            Ok(bytes) => {
                let res = serde_json::from_slice::<ChatMessageResponse>(&bytes);
                match res {
                    Ok(res) => Ok(res),
                    Err(e) => {
                        eprintln!("Failed to deserialize response: {}", e);
                        Err(())
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to read response: {}", e);
                Err(())
            }
        }));

        Ok(std::pin::Pin::from(stream))
    }

    /// Chat message generation.
    /// Returns a `ChatMessageResponse` object
    pub async fn send_chat_messages(
        &self,
        request: ChatMessageRequest,
    ) -> crate::error::Result<ChatMessageResponse> {
        let mut request = request;
        request.stream = false;

        let url = format!("{}api/chat", self.url_str());
        let serialized = serde_json::to_string(&request).map_err(|e| e.to_string())?;

        let res = self
            .reqwest_client
            .post(url)
            .body(serialized)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            return Err(res.text().await.unwrap_or_else(|e| e.to_string()).into());
        }

        let bytes = res.bytes().await.map_err(|e| e.to_string())?;
        let res =
            serde_json::from_slice::<ChatMessageResponse>(&bytes).map_err(|e| e.to_string())?;

        Ok(res)
    }
}

#[cfg(feature = "chat-history")]
impl Ollama {
    #[cfg(feature = "stream")]
    pub async fn send_chat_messages_with_history_stream(
        &mut self,
        mut request: ChatMessageRequest,
        history_id: impl ToString,
    ) -> crate::error::Result<ChatMessageResponseStream> {
        use async_stream::stream;
        use tokio_stream::StreamExt;
        let id_copy = history_id.to_string().clone();

        let mut current_chat_messages = self.get_chat_messages_by_id(id_copy.clone());

        if let Some(message) = request.messages.first() {
            current_chat_messages.push(message.clone());
        }

        // The request is modified to include the current chat messages
        request.messages.clone_from(&current_chat_messages);
        request.stream = true;

        let mut resp_stream: ChatMessageResponseStream =
            self.send_chat_messages_stream(request.clone()).await?;

        let messages_history = self.messages_history.clone();

        let s = stream! {
            let mut result = String::new();

            while let Some(item) = resp_stream.try_next().await.unwrap() {
                let msg_part = item.clone().message.unwrap().content;

                if item.done {
                    if let Some(history) = messages_history.clone() {
                        let mut inner = history.write().unwrap();
                        // Message we sent to AI
                        if let Some(message) = request.messages.last() {
                            inner.add_message(id_copy.clone(), message.clone());
                        }

                        // AI's response
                        inner.add_message(id_copy.clone(), ChatMessage::assistant(result.clone()));
                    }
                } else {
                    result.push_str(&msg_part);
                }

                yield Ok(item);
            }
        };

        Ok(Box::pin(s))
    }

    /// Chat message generation
    /// Returns a `ChatMessageResponse` object
    /// Manages the history of messages for the given `id`
    pub async fn send_chat_messages_with_history(
        &mut self,
        mut request: ChatMessageRequest,
        history_id: impl ToString,
    ) -> crate::error::Result<ChatMessageResponse> {
        // The request is modified to include the current chat messages
        let id_copy = history_id.to_string().clone();
        let mut current_chat_messages = self.get_chat_messages_by_id(id_copy.clone());

        if let Some(message) = request.messages.first() {
            current_chat_messages.push(message.clone());
        }

        // The request is modified to include the current chat messages
        request.messages.clone_from(&current_chat_messages);

        let result = self.send_chat_messages(request.clone()).await;

        if let Ok(result) = result {
            // Message we sent to AI
            if let Some(message) = request.messages.last() {
                self.store_chat_message_by_id(id_copy.clone(), message.clone());
            }
            // Store AI's response in the history
            self.store_chat_message_by_id(id_copy, result.message.clone().unwrap());

            return Ok(result);
        }

        result
    }

    /// Helper function to store chat messages by id
    fn store_chat_message_by_id(&mut self, id: impl ToString, message: ChatMessage) {
        if let Some(messages_history) = self.messages_history.as_mut() {
            messages_history.write().unwrap().add_message(id, message);
        }
    }

    /// Let get existing history with a new message in it
    /// Without impact for existing history
    /// Used to prepare history for request
    fn get_chat_messages_by_id(&mut self, history_id: impl ToString) -> Vec<ChatMessage> {
        let mut new_history = std::sync::Arc::new(std::sync::RwLock::new(MessagesHistory::default()));
        let chat_history = match self.messages_history.as_mut() {
            Some(history) => history,
            None => {
                self.messages_history = Some(new_history.clone());
                &mut new_history
            }
        };
        // Clone the current chat messages to avoid borrowing issues
        // And not to add message to the history if the request fails
        let mut history_instance = chat_history.write().unwrap();
        let chat_history = history_instance
            .messages_by_id
            .entry(history_id.to_string())
            .or_default();

        chat_history.clone()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatMessageResponse {
    /// The name of the model used for the completion.
    pub model: String,
    /// The creation time of the completion, in such format: `2023-08-04T08:52:19.385406455-07:00`.
    pub created_at: String,
    /// The generated chat message.
    pub message: Option<ChatMessage>,
    pub done: bool,
    #[serde(flatten)]
    /// The final data of the completion. This is only present if the completion is done.
    pub final_data: Option<ChatMessageFinalResponseData>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatMessageFinalResponseData {
    /// Time spent generating the response
    pub total_duration: u64,
    /// Number of tokens in the prompt
    pub prompt_eval_count: u16,
    /// Time spent in nanoseconds evaluating the prompt
    pub prompt_eval_duration: u64,
    /// Number of tokens the response
    pub eval_count: u16,
    /// Time in nanoseconds spent generating the response
    pub eval_duration: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub function: ToolCallFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    pub images: Option<Vec<Image>>,
    pub tool_calls: Option<Vec<ToolCall>>,
}

impl ChatMessage {
    pub fn new(role: MessageRole, content: String) -> Self {
        Self {
            role,
            content,
            images: None,
            tool_calls: None,
        }
    }

    pub fn user(content: String) -> Self {
        Self::new(MessageRole::User, content)
    }

    pub fn assistant(content: String) -> Self {
        Self::new(MessageRole::Assistant, content)
    }

    pub fn system(content: String) -> Self {
        Self::new(MessageRole::System, content)
    }

    pub fn with_images(mut self, images: Vec<Image>) -> Self {
        self.images = Some(images);
        self
    }

    pub fn add_image(mut self, image: Image) -> Self {
        if let Some(images) = self.images.as_mut() {
            images.push(image);
        } else {
            self.images = Some(vec![image]);
        }
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageRole {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
    #[serde(rename = "system")]
    System,
}
