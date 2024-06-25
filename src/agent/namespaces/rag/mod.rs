use std::{collections::HashMap, time::Instant};

use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;

use crate::agent::state::SharedState;

use super::{Action, Namespace};

#[derive(Debug, Default, Clone)]
struct Search {}

#[async_trait]
impl Action for Search {
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
    ) -> Result<Option<String>> {
        let query = payload.unwrap();
        let start = Instant::now();
        // TODO: make top_k configurable?
        let docs = state.lock().await.rag_query(&query, 1).await?;

        if !docs.is_empty() {
            println!("\n  {} results in {:?}", docs.len(), start.elapsed());
            for (doc, score) in &docs {
                println!("       * {} ({})", &doc.name, score);
            }
            println!("");

            Ok(Some(format!(
                "Here is some supporting information:\n\n{}",
                docs.iter()
                    .map(|(doc, _)| doc.data.clone())
                    .collect::<Vec<String>>()
                    .join("\n")
            )))
        } else {
            println!(
                "[{}] no results for '{query}' in {:?}",
                "rag".bold(),
                start.elapsed()
            );
            Ok(Some("no documents for this query".to_string()))
        }
    }
}

pub(crate) fn get_namespace() -> Namespace {
    Namespace::new_non_default(
        "Knowledge".to_string(),
        include_str!("ns.prompt").to_string(),
        vec![Box::<Search>::default()],
        None,
    )
}
