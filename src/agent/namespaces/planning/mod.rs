use std::collections::HashMap;

use crate::agent::state::State;
use anyhow::Result;

use super::{Action, Namespace, StorageDescriptor};

// TODO: implement step complete/not complete state and changes

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
            state.get_storage("plan")?.add_untagged(&payload.unwrap());
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
            .del_untagged(payload.unwrap().parse::<usize>()?);
        Ok(Some("step added to the plan".to_string()))
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
    Namespace::new(
        "Planning".to_string(),
        // TODO: improve this - it should be clear to the model that it should deconstruct complex problems in smaller ones using this tool.
        include_str!("ns.prompt").to_string(),
        vec![
            Box::<AddStep>::default(),
            Box::<DeleteStep>::default(),
            Box::<Clear>::default(),
        ],
        Some(vec![StorageDescriptor::untagged("plan")]),
    )
}
