use std::collections::HashMap;

use crate::agent::state::State;
use anyhow::Result;

use super::{Action, Namespace};

#[derive(Debug, Default)]
struct SaveMemory {}

impl Action for SaveMemory {
    fn name(&self) -> &str {
        "save-memory"
    }

    fn description(&self) -> &str {
        "To store a memory:"
    }

    fn add_to_activity(&self) -> bool {
        false
    }

    fn attributes(&self) -> Option<HashMap<String, String>> {
        let mut attributes = HashMap::new();

        attributes.insert("key".to_string(), "my-note".to_string());

        Some(attributes)
    }

    fn example_payload(&self) -> Option<&str> {
        Some("put here the custom data you want to keep for later")
    }

    fn run(
        &self,
        state: &State,
        attributes: Option<HashMap<String, String>>,
        payload: Option<String>,
    ) -> Result<Option<String>> {
        if let Some(attrs) = attributes {
            if let Some(key) = attrs.get("key") {
                if let Some(data) = payload {
                    state.add_memory(key.to_string(), data);
                    return Ok(None);
                }

                return Err(anyhow!("no content specified for save-memory"));
            }

            return Err(anyhow!("no key attribute specified for save-memory"));
        }

        Err(anyhow!("no attributes specified for save-memory"))
    }
}

#[derive(Debug, Default)]
struct DeleteMemory {}

impl Action for DeleteMemory {
    fn name(&self) -> &str {
        "delete-memory"
    }

    fn description(&self) -> &str {
        "To delete a memory you previously stored given its key:"
    }

    fn add_to_activity(&self) -> bool {
        false
    }

    fn attributes(&self) -> Option<HashMap<String, String>> {
        let mut attributes = HashMap::new();

        attributes.insert("key".to_string(), "my-note".to_string());

        Some(attributes)
    }

    fn run(
        &self,
        state: &State,
        attributes: Option<HashMap<String, String>>,
        _: Option<String>,
    ) -> Result<Option<String>> {
        if let Some(attrs) = attributes {
            if let Some(key) = attrs.get("key") {
                return if state.remove_memory(key).is_some() {
                    Ok(None)
                } else {
                    Err(anyhow!("memory '{}' not found", key))
                };
            }

            return Err(anyhow!("no key attribute specified for delete-memory"));
        }

        Err(anyhow!("no attributes specified for delete-memory"))
    }
}

pub(crate) fn get_functions() -> Namespace {
    Namespace::new(
        "Memory".to_string(), 
        "You can use the memory actions to store and retrieve long term information as you work. Use memories often to keep track of important information like your planning, analysis, important web responses, etc.".to_string(),
        vec![Box::<SaveMemory>::default(), Box::<DeleteMemory>::default()],
    )
}
