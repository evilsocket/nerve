use anyhow::Result;

use std::collections::HashMap;

use crate::agent::state::State;

use super::{Action, Group};

#[derive(Debug, Default)]
struct UpdateGoal {}

impl Action for UpdateGoal {
    fn name(&self) -> &str {
        "update-goal"
    }

    fn description(&self) -> &str {
        "When you believe you need a new goal:"
    }

    fn example_payload(&self) -> Option<&str> {
        Some("my new goal")
    }

    fn run(
        &self,
        state: &State,
        _: Option<HashMap<String, String>>,
        payload: Option<String>,
    ) -> Result<Option<String>> {
        state.set_new_goal(payload.unwrap());
        Ok(None)
    }
}

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
        Some("put jere a brief report about why the task is complete")
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

pub(crate) fn get_functions() -> Group {
    Group::new(
        "Task".to_string(),
        "".to_string(),
        vec![
            Box::new(CompleteTask::default()),
            // Box::new(UpdateGoal::default()),
        ],
    )
}
