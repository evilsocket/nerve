use std::collections::HashMap;

use xml::serialize_invocation;

pub(crate) mod xml;

#[derive(Debug, Default, Clone)]
pub struct Invocation {
    pub action: String,
    pub attributes: Option<HashMap<String, String>>,
    pub payload: Option<String>,

    serialized: String,
}

impl Invocation {
    pub fn new(
        action: String,
        attributes: Option<HashMap<String, String>>,
        payload: Option<String>,
    ) -> Self {
        let mut zelf = Self {
            action,
            attributes,
            payload,
            serialized: "".to_string(),
        };
        zelf.serialized = serialize_invocation(&zelf);
        zelf
    }

    pub fn as_serialized_str(&self) -> &str {
        return self.serialized.as_str();
    }

    pub fn is_same(&self, other: &Invocation) -> bool {
        self.serialized == other.serialized
    }
}
