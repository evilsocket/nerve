use std::collections::HashMap;

use crate::agent::state::State;
use anyhow::Result;

use super::{Action, Namespace};

#[derive(Debug, Default)]
struct Gather {}

impl Action for Gather {
    fn name(&self) -> &str {
        "gather-memory"
    }

    fn description(&self) -> &str {
        "To retrieve information by asking a question:"
    }

    fn example_payload(&self) -> Option<&str> {
        Some("put here the question you want to ask")
    }

    fn run(
        &self,
        _: &State,
        _: Option<HashMap<String, String>>,
        payload: Option<String>,
    ) -> Result<Option<String>> {
        if payload.is_none() {
            return Err(anyhow!("no question provided"));
        }

        // TODO: https://api.duckduckgo.com/?q=what%20is%20god&format=json
        Ok(None)
    }
}

pub(crate) fn get_namespace() -> Namespace {
    Namespace::new(
        "Knowledge".to_string(),
        "You can use the knowledge actions to retrieve information you don't already have."
            .to_string(),
        vec![Box::<Gather>::default()],
        None,
    )
}
