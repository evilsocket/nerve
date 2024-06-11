use anyhow::Result;

use std::collections::HashMap;

use crate::agent::state::State;

use super::{Action, Namespace};

#[derive(Debug, Default)]
struct CompleteTask {}

impl Action for CompleteTask {
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
        state.on_complete(payload)?;
        Ok(None)
    }
}

pub(crate) fn get_functions() -> Namespace {
    Namespace::new(
        "Task".to_string(),
        "Use these actions to set the task as completed or update your current goal.".to_string(),
        vec![Box::<CompleteTask>::default()],
    )
}
