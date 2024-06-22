use std::collections::HashMap;

use anyhow::Result;

use super::{Action, Namespace, StorageDescriptor};
use crate::agent::state::State;

#[derive(Debug, Default)]
struct AddStep {}

impl Action for AddStep {
    fn name(&self) -> &str {
        "add-plan-step"
    }

    fn description(&self) -> &str {
        include_str!("add.prompt")
    }

    fn example_payload(&self) -> Option<&str> {
        Some("complete the task")
    }

    fn run(
        &self,
        state: &State,
        _: Option<HashMap<String, String>>,
        payload: Option<String>,
    ) -> Result<Option<String>> {
        if payload.is_none() {
            Err(anyhow!("no step description provided"))
        } else {
            state.get_storage("plan")?.add_completion(&payload.unwrap());
            Ok(Some("step added to the plan".to_string()))
        }
    }
}

#[derive(Debug, Default)]
struct DeleteStep {}

impl Action for DeleteStep {
    fn name(&self) -> &str {
        "delete-plan-step"
    }

    fn description(&self) -> &str {
        include_str!("delete.prompt")
    }

    fn example_payload(&self) -> Option<&str> {
        Some("2")
    }

    fn run(
        &self,
        state: &State,
        _: Option<HashMap<String, String>>,
        payload: Option<String>,
    ) -> Result<Option<String>> {
        if payload.is_none() {
            return Err(anyhow!("no position provided"));
        }

        state
            .get_storage("plan")?
            .del_completion(payload.unwrap().parse::<usize>()?);
        Ok(Some("step removed from the plan".to_string()))
    }
}

#[derive(Debug, Default)]
struct SetComplete {}

impl Action for SetComplete {
    fn name(&self) -> &str {
        "set-step-completed"
    }

    fn description(&self) -> &str {
        include_str!("set-complete.prompt")
    }

    fn example_payload(&self) -> Option<&str> {
        Some("2")
    }

    fn run(
        &self,
        state: &State,
        _: Option<HashMap<String, String>>,
        payload: Option<String>,
    ) -> Result<Option<String>> {
        if payload.is_none() {
            return Err(anyhow!("no position provided"));
        }

        let pos = payload.unwrap().parse::<usize>()?;
        if state.get_storage("plan")?.set_complete(pos).is_some() {
            Ok(Some(format!("step {} marked as completed", pos)))
        } else {
            Err(anyhow!("no plan step at position {}", pos))
        }
    }
}

#[derive(Debug, Default)]
struct SetIncomplete {}

impl Action for SetIncomplete {
    fn name(&self) -> &str {
        "set-step-incomplete"
    }

    fn description(&self) -> &str {
        include_str!("set-incomplete.prompt")
    }

    fn example_payload(&self) -> Option<&str> {
        Some("2")
    }

    fn run(
        &self,
        state: &State,
        _: Option<HashMap<String, String>>,
        payload: Option<String>,
    ) -> Result<Option<String>> {
        if payload.is_none() {
            return Err(anyhow!("no position provided"));
        }

        let pos = payload.unwrap().parse::<usize>()?;
        if state.get_storage("plan")?.set_incomplete(pos).is_some() {
            Ok(Some(format!("step {} marked as incomplete", pos)))
        } else {
            Err(anyhow!("no plan step at position {}", pos))
        }
    }
}

#[derive(Debug, Default)]
struct Clear {}

impl Action for Clear {
    fn name(&self) -> &str {
        "clear-plan"
    }

    fn description(&self) -> &str {
        include_str!("clear.prompt")
    }

    fn run(
        &self,
        state: &State,
        _: Option<HashMap<String, String>>,
        _: Option<String>,
    ) -> Result<Option<String>> {
        state.get_storage("plan")?.clear();
        Ok(Some("plan cleared".to_string()))
    }
}

pub(crate) fn get_namespace() -> Namespace {
    Namespace::new_default(
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
