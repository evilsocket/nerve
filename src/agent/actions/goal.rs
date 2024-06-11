use anyhow::Result;

use std::collections::HashMap;

use crate::agent::state::State;

use super::{Action, Namespace};

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
        Ok(Some("goal updated".to_string()))
    }
}

pub(crate) fn get_functions() -> Namespace {
    Namespace::new(
        "Goal".to_string(),
        "Use these actions to update your current goal.".to_string(),
        vec![Box::<UpdateGoal>::default()],
    )
}
