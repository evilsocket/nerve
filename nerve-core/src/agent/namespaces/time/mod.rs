use anyhow::Result;
use async_trait::async_trait;

use std::{collections::HashMap, time::Duration};

use super::{Action, Namespace, StorageDescriptor};
use crate::agent::state::SharedState;

#[derive(Debug, Default, Clone)]
struct Wait {}

#[async_trait]
impl Action for Wait {
    fn name(&self) -> &str {
        "wait"
    }

    fn description(&self) -> &str {
        include_str!("wait.prompt")
    }

    fn example_payload(&self) -> Option<&str> {
        Some("5")
    }

    async fn run(
        &self,
        _: SharedState,
        _: Option<HashMap<String, String>>,
        payload: Option<String>,
    ) -> Result<Option<String>> {
        let secs = payload.unwrap().parse::<u64>()?;

        log::info!("sleeping for {secs} seconds ...");

        tokio::time::sleep(Duration::from_secs(secs)).await;

        Ok(None)
    }
}

pub fn get_namespace() -> Namespace {
    Namespace::new_default(
        "Time".to_string(),
        "".to_string(),
        vec![Box::<Wait>::default()],
        Some(vec![StorageDescriptor::time("time")]),
    )
}
