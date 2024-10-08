use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::generation::{options::GenerationOptions, parameters::FormatType};

use super::ChatMessage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunctionParameterProperty {
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub the_type: String,
    pub description: String,
    // `enum` is optional and can be a list of strings
    #[serde(rename(serialize = "enum", deserialize = "enum"))]
    pub an_enum: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunctionParameters {
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub the_type: String,
    pub required: Vec<String>,
    pub properties: HashMap<String, ToolFunctionParameterProperty>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    pub description: String,
    pub parameters: ToolFunctionParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub the_type: String,
    pub function: ToolFunction,
}

/// A chat message request to Ollama.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageRequest {
    #[serde(rename = "model")]
    pub model_name: String,
    pub messages: Vec<ChatMessage>,
    pub options: Option<GenerationOptions>,
    pub template: Option<String>,
    pub format: Option<FormatType>,
    pub tools: Vec<Tool>,
    pub stream: bool,
}

impl ChatMessageRequest {
    pub fn new(model_name: String, messages: Vec<ChatMessage>) -> Self {
        Self {
            model_name,
            messages,
            options: None,
            template: None,
            format: None,
            tools: vec![],
            // Stream value will be overwritten by Ollama::send_chat_messages_stream() and Ollama::send_chat_messages() methods
            stream: false,
        }
    }

    /// Additional model parameters listed in the documentation for the Modelfile
    pub fn options(mut self, options: GenerationOptions) -> Self {
        self.options = Some(options);
        self
    }

    /// The full prompt or prompt template (overrides what is defined in the Modelfile)
    pub fn template(mut self, template: String) -> Self {
        self.template = Some(template);
        self
    }

    pub fn tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = tools;
        self
    }

    // The format to return a response in. Currently the only accepted value is `json`
    pub fn format(mut self, format: FormatType) -> Self {
        self.format = Some(format);
        self
    }
}
