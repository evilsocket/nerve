use std::collections::HashMap;

use anyhow::Result;

use super::{Action, Namespace};
use crate::agent::state::State;

#[derive(Debug, Default)]
struct Complete {}

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
        include_str!("impossible.prompt")
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
        include_str!("ns.prompt").to_string(),
        vec![Box::<Complete>::default(), Box::<Impossible>::default()],
        None,
    )
}
