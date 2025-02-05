use serde::{Deserialize, Serialize};

pub mod audio;
pub mod chat;
pub mod completions;
pub mod embeddings;
pub mod images;
pub mod models;

// Models API
const MODELS_LIST: &str = "models";
const MODELS_RETRIEVE: &str = "models/";
// Completions API
const COMPLETION_CREATE: &str = "completions";
// Chat API
const CHAT_COMPLETION_CREATE: &str = "chat/completions";
// Images API
const IMAGES_CREATE: &str = "images/generations";
const IMAGES_EDIT: &str = "images/edits";
const IMAGES_VARIATIONS: &str = "images/variations";
// Embeddings API
const EMBEDDINGS_CREATE: &str = "embeddings";
// Audio API
const AUDIO_TRANSCRIPTION_CREATE: &str = "audio/transcriptions";
const AUDIO_TRANSLATIONS_CREATE: &str = "audio/translations";

#[derive(Debug, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Choice {
    pub text: Option<String>,
    pub index: u32,
    pub logprobs: Option<String>,
    pub finish_reason: Option<String>,
    pub message: Option<Message>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Function {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolCall {
    pub id: String,
    pub function: Function,
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub the_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageUrl {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageContent {
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub the_type: String,
    pub image_url: ImageUrl,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(serialize = "content", deserialize = "content"))]
    pub image_content: Option<Vec<ImageContent>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

impl Message {
    pub fn text(str: &str, role: Role) -> Self {
        Self {
            role,
            content: Some(str.to_string()),
            image_content: None,
            tool_calls: None,
        }
    }

    pub fn image(data: &str, mime_type: &str, role: Role) -> Self {
        Self {
            role,
            content: None,
            image_content: Some(vec![ImageContent {
                image_url: ImageUrl {
                    url: if data.starts_with("http://") || data.starts_with("https://") {
                        data.to_string()
                    } else {
                        format!("data:{};base64,{}", mime_type, data)
                    },
                },
                the_type: "image_url".to_string(),
            }]),
            tool_calls: None,
        }
    }
}

impl Clone for Message {
    fn clone(&self) -> Self {
        Self {
            role: self.role.clone(),
            content: self.content.clone(),
            image_content: self.image_content.clone(),
            tool_calls: self.tool_calls.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    Assistant,
    User,
}

impl Clone for Role {
    fn clone(&self) -> Self {
        match self {
            Self::System => Self::System,
            Self::Assistant => Self::Assistant,
            Self::User => Self::User,
        }
    }
}
