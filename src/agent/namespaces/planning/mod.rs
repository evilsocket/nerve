use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;

use super::{Tool, ToolOutput, Namespace, StorageDescriptor};
use crate::agent::state::SharedState;

#[derive(Debug, Default, Clone)]
struct AddStep {}

#[async_trait]
impl Tool for AddStep {
    fn name(&self) -> &str {
        "add_plan_step"
    }

    fn description(&self) -> &str {
        include_str!("add.prompt")
    }

    fn example_payload(&self) -> Option<&str> {
        Some("complete the task")
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
            .get_storage_mut("plan")?
            .add_completion(&payload.unwrap());
        Ok(Some("step added to the plan".into()))
    }
}

#[derive(Debug, Default, Clone)]
struct DeleteStep {}

#[async_trait]
impl Tool for DeleteStep {
    fn name(&self) -> &str {
        "delete_plan_step"
    }

    fn description(&self) -> &str {
        include_str!("delete.prompt")
    }

    fn example_payload(&self) -> Option<&str> {
        Some("2")
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
            .get_storage_mut("plan")?
            .del_completion(payload.unwrap().parse::<usize>()?);
        Ok(Some("step removed from the plan".into()))
    }
}

#[derive(Debug, Default, Clone)]
struct SetComplete {}

#[async_trait]
impl Tool for SetComplete {
    fn name(&self) -> &str {
        "set_step_completed"
    }

    fn description(&self) -> &str {
        include_str!("set-complete.prompt")
    }

    fn example_payload(&self) -> Option<&str> {
        Some("2")
    }

    async fn run(
        &self,
        state: SharedState,
        _: Option<HashMap<String, String>>,
        payload: Option<String>,
    ) -> Result<Option<ToolOutput>> {
        let pos = payload.unwrap().parse::<usize>()?;
        if state
            .lock()
            .await
            .get_storage_mut("plan")?
            .set_complete(pos)
            .is_some()
        {
            Ok(Some(format!("step {} marked as completed", pos).into()))
        } else {
            Err(anyhow!("no plan step at position {}", pos))
        }
    }
}

#[derive(Debug, Default, Clone)]
struct SetIncomplete {}

#[async_trait]
impl Tool for SetIncomplete {
    fn name(&self) -> &str {
        "set_step_incomplete"
    }

    fn description(&self) -> &str {
        include_str!("set-incomplete.prompt")
    }

    fn example_payload(&self) -> Option<&str> {
        Some("2")
    }

    async fn run(
        &self,
        state: SharedState,
        _: Option<HashMap<String, String>>,
        payload: Option<String>,
    ) -> Result<Option<ToolOutput>> {
        let pos = payload.unwrap().parse::<usize>()?;
        if state
            .lock()
            .await
            .get_storage_mut("plan")?
            .set_incomplete(pos)
            .is_some()
        {
            Ok(Some(format!("step {} marked as incomplete", pos).into()))
        } else {
            Err(anyhow!("no plan step at position {}", pos))
        }
    }
}

#[derive(Debug, Default, Clone)]
struct Clear {}

#[async_trait]
impl Tool for Clear {
    fn name(&self) -> &str {
        "clear_plan"
    }

    fn description(&self) -> &str {
        include_str!("clear.prompt")
    }

    async fn run(
        &self,
        state: SharedState,
        _: Option<HashMap<String, String>>,
        _: Option<String>,
    ) -> Result<Option<ToolOutput>> {
        state.lock().await.get_storage_mut("plan")?.clear();
        Ok(Some("plan cleared".into()))
    }
}

pub fn get_namespace() -> Namespace {
    Namespace::new_non_default(
        "Planning".to_string(),
        include_str!("ns.prompt").to_string(),
        vec![
            Box::<AddStep>::default(),
            Box::<DeleteStep>::default(),
            Box::<SetComplete>::default(),
            Box::<SetIncomplete>::default(),
            Box::<Clear>::default(),
        ],
        Some(vec![StorageDescriptor::completion("plan")]),
    )
}
