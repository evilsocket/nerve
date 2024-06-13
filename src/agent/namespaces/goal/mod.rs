use anyhow::Result;

use std::collections::HashMap;

use crate::agent::state::State;

use super::{Action, Namespace, StorageDescriptor};

#[derive(Debug, Default)]
struct UpdateGoal {}

impl Action for UpdateGoal {
    fn name(&self) -> &str {
        "update-goal"
    }

    fn description(&self) -> &str {
        include_str!("update.prompt")
    }

    fn example_payload(&self) -> Option<&str> {
        Some("your new goal")
    }

    fn run(
        &self,
        state: &State,
        _: Option<HashMap<String, String>>,
        payload: Option<String>,
    ) -> Result<Option<String>> {
        state
            .get_storage("goal")?
            .set_current(payload.as_ref().unwrap(), true);
        Ok(Some("goal updated".to_string()))
    }
}

pub(crate) fn get_namespace() -> Namespace {
    Namespace::new(
        "Goal".to_string(),
        include_str!("ns.prompt").to_string(),
        vec![Box::<UpdateGoal>::default()],
        Some(vec![StorageDescriptor::previous_current("goal")]),
    )
}
