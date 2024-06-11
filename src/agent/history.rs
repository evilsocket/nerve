use anyhow::Result;

use super::{generator::Message, Invocation};

#[derive(Debug, Clone, Default)]
pub struct Execution {
    pub invocation: Invocation,
    pub result: Option<String>,
    pub error: Option<String>,
}

impl Execution {
    pub fn new(invocation: Invocation, result: Option<String>, error: Option<String>) -> Self {
        Self {
            invocation,
            result,
            error,
        }
    }

    pub fn to_messages(&self) -> Vec<Message> {
        let mut messages = vec![];

        messages.push(Message::Agent(self.invocation.as_xml().to_string()));
        messages.push(Message::User(if let Some(err) = &self.error {
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
