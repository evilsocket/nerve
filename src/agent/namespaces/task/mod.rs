use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;

use super::{Action, Namespace};
use crate::agent::state::SharedState;

#[derive(Debug, Default, Clone)]
struct Complete {}

#[async_trait]
impl Action for Complete {
    fn name(&self) -> &str {
        "task-complete"
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
    ) -> Result<Option<String>> {
        state.lock().await.on_complete(false, payload)?;
        Ok(None)
    }
}

#[derive(Debug, Default, Clone)]
struct Impossible {}

#[async_trait]
impl Action for Impossible {
    fn name(&self) -> &str {
        "task-impossible"
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
    ) -> Result<Option<String>> {
        state.lock().await.on_complete(true, payload)?;
        Ok(None)
    }
}

pub(crate) fn get_namespace() -> Namespace {
    Namespace::new_default(
        "Task".to_string(),
        include_str!("ns.prompt").to_string(),
        vec![Box::<Complete>::default(), Box::<Impossible>::default()],
        None,
    )
}
