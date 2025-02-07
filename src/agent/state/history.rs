use anyhow::Result;

use crate::agent::{generator::Message, namespaces::ActionOutput, serialization, ToolCall};

#[derive(Debug, Clone, Default)]
pub struct Execution {
    // unparsed response caused an error
    response: Option<String>,
    // parsed tool call
    tool_call: Option<ToolCall>,

    result: Option<ActionOutput>,
    error: Option<String>,
}

impl Execution {
    pub fn with_unparsed_response(response: &str, error: String) -> Self {
        Self {
            tool_call: None,
            response: Some(response.to_string()),
            result: None,
            error: Some(error),
        }
    }

    pub fn with_feedback(message: String) -> Self {
        Self {
            tool_call: None,
            response: None,
            result: Some(message.into()),
            error: None,
        }
    }

    pub fn with_error(tool_call: ToolCall, error: String) -> Self {
        Self {
            tool_call: Some(tool_call),
            response: None,
            result: None,
            error: Some(error),
        }
    }

    pub fn with_result(tool_call: ToolCall, result: Option<ActionOutput>) -> Self {
        Self {
            tool_call: Some(tool_call),
            response: None,
            result,
            error: None,
        }
    }

    pub fn to_messages(&self, serializer: &serialization::Strategy) -> Vec<Message> {
        let mut messages = vec![];

        if let Some(response) = self.response.as_ref() {
            messages.push(Message::Agent {
                content: response.to_string(),
                tool_call: None,
            });
        } else if let Some(tool_call) = self.tool_call.as_ref() {
            messages.push(Message::Agent {
                content: serializer.serialize_tool_call(tool_call),
                tool_call: Some(tool_call.clone()),
            });
        }

        messages.push(Message::Feedback {
            result: if let Some(err) = &self.error {
                ActionOutput::text(format!("ERROR: {err}"))
            } else if let Some(out) = &self.result {
                out.clone()
            } else {
                ActionOutput::text("")
            },
            tool_call: self.tool_call.clone(),
        });

        messages
    }
}

#[derive(Debug, Clone)]
pub struct History(Vec<Execution>);

impl History {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn to_chat_history(&self, serializer: &serialization::Strategy) -> Result<Vec<Message>> {
        let mut history = vec![];

        for entry in self.0.iter() {
            history.extend(entry.to_messages(serializer));
        }

        Ok(history)
    }
}

impl std::ops::Deref for History {
    type Target = Vec<Execution>;
    fn deref(&self) -> &Vec<Execution> {
        &self.0
    }
}

impl std::ops::DerefMut for History {
    fn deref_mut(&mut self) -> &mut Vec<Execution> {
        &mut self.0
    }
}
