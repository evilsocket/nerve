use std::hash::Hash;

use serde::{Deserialize, Serialize};

/// 1:1 Mapping for Message Object used in the `messages` field groq completion API.
///
/// Refer to [the official documentations](https://console.groq.com/docs/api-reference#chat-create)
/// for more details
///
#[derive(Debug, Serialize, Clone, Hash)]
#[serde(untagged)]
pub enum Message {
    SystemMessage {
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        role: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_call_id: Option<String>,
    },
    UserMessage {
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        role: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_call_id: Option<String>,
    },
    AssistantMessage {
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        role: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_calls: Option<Vec<ToolCall>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_call_id: Option<String>,
    },
    ToolMessage {
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        role: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_call_id: Option<String>,
    },
}

#[derive(Debug, Deserialize, Serialize, Clone, Hash)]
pub struct ToolCall {
    pub id: Option<String>,
    #[serde(rename(serialize = "type"))]
    pub tool_type: Option<String>,
    pub function: AssistantFunc,
}

#[derive(Debug, Deserialize, Serialize, Clone, Hash)]
pub struct AssistantFunc {
    pub arguments: Option<String>,
    pub name: Option<String>,
}
