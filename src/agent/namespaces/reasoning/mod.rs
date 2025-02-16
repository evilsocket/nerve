use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;

use super::{Namespace, StorageDescriptor, Tool, ToolOutput};
use crate::agent::state::SharedState;

#[derive(Debug, Default, Clone)]
struct Think {}

#[async_trait]
impl Tool for Think {
    fn name(&self) -> &str {
        "think"
    }

    fn description(&self) -> &str {
        include_str!("think.prompt")
    }

    fn example_payload(&self) -> Option<&str> {
        Some(include_str!("think_example.prompt"))
    }

    async fn run(
        &self,
        state: SharedState,
        _: Option<HashMap<String, String>>,
        payload: Option<String>,
    ) -> Result<Option<ToolOutput>> {
        state
            .lock()
            .await
            .get_storage_mut("reasoning")?
            .add_text(payload.unwrap().as_str())
            .await;

        Ok(None)
    }
}

#[derive(Debug, Default, Clone)]
struct ClearThoughts {}

#[async_trait]
impl Tool for ClearThoughts {
    fn name(&self) -> &str {
        "clear_thoughts"
    }

    fn description(&self) -> &str {
        include_str!("clear_thoughts.prompt")
    }

    async fn run(
        &self,
        state: SharedState,
        _: Option<HashMap<String, String>>,
        _: Option<String>,
    ) -> Result<Option<ToolOutput>> {
        state
            .lock()
            .await
            .get_storage_mut("reasoning")?
            .clear()
            .await;

        Ok(None)
    }
}

pub fn get_namespace() -> Namespace {
    Namespace::new_non_default(
        "Reasoning".to_string(),
        include_str!("ns.prompt").to_string(),
        vec![Box::<Think>::default(), Box::<ClearThoughts>::default()],
        Some(vec![StorageDescriptor::text("reasoning")]),
    )
}
