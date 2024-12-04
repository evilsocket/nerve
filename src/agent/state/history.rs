use anyhow::Result;

use crate::agent::{generator::Message, serialization, Invocation};

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

    pub fn to_messages(&self, serializer: &serialization::Strategy) -> Vec<Message> {
        let mut messages = vec![];

        if let Some(response) = self.response.as_ref() {
            messages.push(Message::Agent(response.to_string(), None));
        } else if let Some(invocation) = self.invocation.as_ref() {
            messages.push(Message::Agent(
                serializer.serialize_invocation(invocation),
                Some(invocation.clone()),
            ));
        }

        messages.push(Message::Feedback(
            if let Some(err) = &self.error {
                format!("ERROR: {err}")
            } else if let Some(out) = &self.result {
                out.to_string()
            } else {
                "".to_string()
            },
            self.invocation.clone(),
        ));

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
