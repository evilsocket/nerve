use std::collections::HashMap;

use anyhow::Result;
use colored::Colorize;

use super::{Action, Namespace, StorageDescriptor};
use crate::agent::state::State;

#[derive(Debug, Default)]
struct SaveMemory {}

impl Action for SaveMemory {
    fn name(&self) -> &str {
        "save-memory"
    }

    fn description(&self) -> &str {
        include_str!("save.prompt")
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
        let attrs = attributes.unwrap();
        let key = attrs.get("key").unwrap();

        state
            .get_storage("memories")?
            .add_tagged(key, payload.unwrap().as_str());

        Ok(Some("memory saved".to_string()))
    }
}

#[derive(Debug, Default)]
struct DeleteMemory {}

impl Action for DeleteMemory {
    fn name(&self) -> &str {
        "delete-memory"
    }

    fn description(&self) -> &str {
        include_str!("delete.prompt")
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
        let attrs = attributes.unwrap();
        let key = attrs.get("key").unwrap();
        if state.get_storage("memories")?.del_tagged(key).is_some() {
            Ok(Some("memory deleted".to_string()))
        } else {
            Err(anyhow!("memory '{}' not found", key))
        }
    }
}

#[derive(Debug, Default)]
struct RecallMemory {}

impl Action for RecallMemory {
    fn name(&self) -> &str {
        "recall-memory"
    }

    fn description(&self) -> &str {
        include_str!("recall.prompt")
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
        let attrs = attributes.unwrap();
        let key = attrs.get("key").unwrap();

        if let Some(memory) = state.get_storage("memories")?.get_tagged(key) {
            println!("<{}> recalling {}", "memories".bold(), key);
            Ok(Some(memory))
        } else {
            eprintln!("<{}> memory {} does not exist", "memories".bold(), key);
            Err(anyhow!("memory '{}' not found", key))
        }
    }
}

pub(crate) fn get_namespace() -> Namespace {
    Namespace::new_default(
        "Memory".to_string(),
        include_str!("ns.prompt").to_string(),
        vec![Box::<SaveMemory>::default(), Box::<DeleteMemory>::default()],
        Some(vec![StorageDescriptor::tagged("memories")]),
    )
}
