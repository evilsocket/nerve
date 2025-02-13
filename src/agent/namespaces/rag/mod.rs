use std::{collections::HashMap, time::Instant};

use anyhow::Result;
use async_trait::async_trait;

use crate::agent::state::SharedState;

use super::{Tool, ToolOutput, Namespace};

#[derive(Debug, Default, Clone)]
struct Search {}

#[async_trait]
impl Tool for Search {
    fn name(&self) -> &str {
        "search"
    }

    fn description(&self) -> &str {
        include_str!("search.prompt")
    }

    fn example_payload(&self) -> Option<&str> {
        Some("what is the biggest city in the world?")
    }

    async fn run(
        &self,
        state: SharedState,
        _: Option<HashMap<String, String>>,
        payload: Option<String>,
    ) -> Result<Option<ToolOutput>> {
        let query = payload.unwrap();
        let start = Instant::now();
        // TODO: make top_k configurable?
        let mut docs = state.lock().await.rag_query(&query, 1).await?;

        if !docs.is_empty() {
            log::debug!(
                "rag search for '{}': {} results in {:?}",
                query,
                docs.len(),
                start.elapsed()
            );

            Ok(Some(
                format!(
                    "Here is some supporting information:\n\n{}",
                    docs.iter_mut()
                        .map(|(doc, _)| doc.get_data().unwrap().to_string())
                        .collect::<Vec<String>>()
                        .join("\n")
                )
                .into(),
            ))
        } else {
            log::debug!(
                "search: no results for query '{query}' in {:?}",
                start.elapsed()
            );
            Ok(Some("no documents for this query".into()))
        }
    }
}

pub fn get_namespace() -> Namespace {
    Namespace::new_non_default(
        "Knowledge".to_string(),
        include_str!("ns.prompt").to_string(),
        vec![Box::<Search>::default()],
        None,
    )
}
