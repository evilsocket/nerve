use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;

use super::{Action, Namespace, StorageDescriptor};
use crate::agent::state::SharedState;

#[derive(Debug, Default, Clone)]
struct SaveMemory {}

#[async_trait]
impl Action for SaveMemory {
    fn name(&self) -> &str {
        "save_memory"
    }

    fn description(&self) -> &str {
        include_str!("save.prompt")
    }

    fn example_attributes(&self) -> Option<HashMap<String, String>> {
        let mut attributes = HashMap::new();

        attributes.insert("key".to_string(), "my-note".to_string());

        Some(attributes)
    }

    fn example_payload(&self) -> Option<&str> {
        Some("put here the custom data you want to keep for later")
    }

    async fn run(
        &self,
        state: SharedState,
        attributes: Option<HashMap<String, String>>,
        payload: Option<String>,
    ) -> Result<Option<String>> {
        let attrs = attributes.unwrap();
        let key = attrs.get("key").unwrap();

        state
            .lock()
            .await
            .get_storage_mut("memories")?
            .add_tagged(key, payload.unwrap().as_str());

        Ok(Some("memory saved".to_string()))
    }
}

#[derive(Debug, Default, Clone)]
struct DeleteMemory {}

#[async_trait]
impl Action for DeleteMemory {
    fn name(&self) -> &str {
        "delete_memory"
    }

    fn description(&self) -> &str {
        include_str!("delete.prompt")
    }

    fn example_attributes(&self) -> Option<HashMap<String, String>> {
        let mut attributes = HashMap::new();

        attributes.insert("key".to_string(), "my-note".to_string());

        Some(attributes)
    }

    async fn run(
        &self,
        state: SharedState,
        attributes: Option<HashMap<String, String>>,
        _: Option<String>,
    ) -> Result<Option<String>> {
        let attrs = attributes.unwrap();
        let key = attrs.get("key").unwrap();
        if state
            .lock()
            .await
            .get_storage_mut("memories")?
            .del_tagged(key)
            .is_some()
        {
            Ok(Some("memory deleted".to_string()))
        } else {
            Err(anyhow!("memory '{}' not found", key))
        }
    }
}

#[derive(Debug, Default, Clone)]
struct RecallMemory {}

#[async_trait]
impl Action for RecallMemory {
    fn name(&self) -> &str {
        "recall_memory"
    }

    fn description(&self) -> &str {
        include_str!("recall.prompt")
    }

    fn example_attributes(&self) -> Option<HashMap<String, String>> {
        let mut attributes = HashMap::new();

        attributes.insert("key".to_string(), "my-note".to_string());

        Some(attributes)
    }

    async fn run(
        &self,
        state: SharedState,
        attributes: Option<HashMap<String, String>>,
        _: Option<String>,
    ) -> Result<Option<String>> {
        let attrs = attributes.unwrap();
        let key = attrs.get("key").unwrap();

        if let Some(memory) = state.lock().await.get_storage("memories")?.get_tagged(key) {
            Ok(Some(memory))
        } else {
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
