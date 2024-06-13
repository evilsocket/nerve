use anyhow::Result;

use crate::agent::{model::Message, parsing::Invocation};

#[derive(Debug, Clone, Default)]
pub struct Execution {
    // unparsed response caused an error
    response: Option<String>,
    // parsed invocation
    invocation: Option<Invocation>,

    result: Option<String>,
    error: Option<String>,
}

impl Execution {
    pub fn with_unparsed_response(response: &str, error: String) -> Self {
        Self {
            invocation: None,
            response: Some(response.to_string()),
            result: None,
            error: Some(error),
        }
    }

    pub fn with_error(invocation: Invocation, error: String) -> Self {
        Self {
            invocation: Some(invocation),
            response: None,
            result: None,
            error: Some(error),
        }
    }

    pub fn with_result(invocation: Invocation, result: Option<String>) -> Self {
        Self {
            invocation: Some(invocation),
            response: None,
            result,
            error: None,
        }
    }

    pub fn to_messages(&self) -> Vec<Message> {
        let mut messages = vec![];

        if let Some(response) = self.response.as_ref() {
            messages.push(Message::Agent(response.to_string()));
        } else if let Some(invocation) = self.invocation.as_ref() {
            messages.push(Message::Agent(invocation.as_xml().to_string()));
        }

        messages.push(Message::Feedback(if let Some(err) = &self.error {
            format!("ERROR: {err}")
        } else if let Some(out) = &self.result {
            out.to_string()
        } else {
            "".to_string()
        }));

        messages
    }
}

#[derive(Debug, Clone)]
pub struct History(Vec<Execution>);

impl History {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn to_chat_history(&self, max: usize) -> Result<Vec<Message>> {
        let mut history = vec![];
        let latest = if self.0.len() > max {
            self.0[self.0.len() - max..].to_vec()
        } else {
            self.0.to_vec().to_vec()
        };

        for entry in latest {
            history.extend(entry.to_messages());
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
