use anyhow::Result;

use std::collections::HashMap;

use crate::agent::state::State;

use super::{Action, Namespace};

#[derive(Debug, Default)]
struct Complete {}

impl Action for Complete {
    fn name(&self) -> &str {
        "task-complete"
    }

    fn description(&self) -> &str {
        "When your objective has been reached:"
    }

    fn example_payload(&self) -> Option<&str> {
        Some("a brief report about why the task is complete")
    }

    fn run(
        &self,
        state: &State,
        _: Option<HashMap<String, String>>,
        payload: Option<String>,
    ) -> Result<Option<String>> {
        state.on_complete(false, payload)?;
        Ok(None)
    }
}

#[derive(Debug, Default)]
struct Impossible {}

impl Action for Impossible {
    fn name(&self) -> &str {
        "task-impossible"
    }

    fn description(&self) -> &str {
        "If you determine that the given goal or task is impossible given the information you have:"
    }

    fn example_payload(&self) -> Option<&str> {
        Some("a brief report about why the task is impossible")
    }

    fn run(
        &self,
        state: &State,
        _: Option<HashMap<String, String>>,
        payload: Option<String>,
    ) -> Result<Option<String>> {
        state.on_complete(true, payload)?;
        Ok(None)
    }
}

pub(crate) fn get_namespace() -> Namespace {
    Namespace::new(
        "Task".to_string(),
        "Use these actions to set the task as completed.".to_string(),
        vec![Box::<Complete>::default(), Box::<Impossible>::default()],
        None,
    )
}
