use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;

use super::{Namespace, Tool, ToolOutput};
use crate::agent::state::SharedState;

#[derive(Debug, Default, Clone)]
struct Complete {}

#[async_trait]
impl Tool for Complete {
    fn name(&self) -> &str {
        "task_complete"
    }

    fn description(&self) -> &str {
        include_str!("complete.prompt")
    }

    fn example_payload(&self) -> Option<&str> {
        Some("a brief report about why the task is complete")
    }

    async fn run(
        &self,
        state: SharedState,
        _: Option<HashMap<String, String>>,
        payload: Option<String>,
    ) -> Result<Option<ToolOutput>> {
        state.lock().await.on_complete(false, payload).await?;
        Ok(None)
    }
}

#[derive(Debug, Default, Clone)]
struct Impossible {}

#[async_trait]
impl Tool for Impossible {
    fn name(&self) -> &str {
        "task_impossible"
    }

    fn description(&self) -> &str {
        include_str!("impossible.prompt")
    }

    fn example_payload(&self) -> Option<&str> {
        Some("a brief report about why the task is impossible")
    }

    async fn run(
        &self,
        state: SharedState,
        _: Option<HashMap<String, String>>,
        payload: Option<String>,
    ) -> Result<Option<ToolOutput>> {
        state.lock().await.on_complete(true, payload).await?;
        Ok(None)
    }
}

pub fn get_namespace() -> Namespace {
    Namespace::new_default(
        "Task".to_string(),
        include_str!("ns.prompt").to_string(),
        vec![Box::<Complete>::default(), Box::<Impossible>::default()],
        None,
    )
}
