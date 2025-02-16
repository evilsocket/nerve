use anyhow::Result;
use async_trait::async_trait;

use std::collections::HashMap;

use super::{Namespace, StorageDescriptor, Tool, ToolOutput};
use crate::agent::state::SharedState;

#[derive(Debug, Default, Clone)]
struct UpdateGoal {}

#[async_trait]
impl Tool for UpdateGoal {
    fn name(&self) -> &str {
        "update_goal"
    }

    fn description(&self) -> &str {
        include_str!("update.prompt")
    }

    fn example_payload(&self) -> Option<&str> {
        Some("your new goal")
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
            .get_storage_mut("goal")?
            .set_current(payload.as_ref().unwrap())
            .await;
        Ok(Some("goal updated".into()))
    }
}

pub fn get_namespace() -> Namespace {
    Namespace::new_non_default(
        "Goal".to_string(),
        include_str!("ns.prompt").to_string(),
        vec![Box::<UpdateGoal>::default()],
        Some(vec![StorageDescriptor::previous_current("goal")]),
    )
}
