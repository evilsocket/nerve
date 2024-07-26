use reqwest;

use scraper::{Html, Selector};
use std::error::Error;

use crate::generation::functions::tools::Tool;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    title: String,
    link: String,
    snippet: String,
}

pub struct DDGSearcher {
    pub client: reqwest::Client,
    pub base_url: String,
}

impl Default for DDGSearcher {
    fn default() -> Self {
        Self::new()
    }
}

impl DDGSearcher {
    pub fn new() -> Self {
        DDGSearcher {
            client: reqwest::Client::new(),
            base_url: "https://duckduckgo.com".to_string(),
        }
    }

    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>, Box<dyn Error>> {
        let url = format!("{}/html/?q={}", self.base_url, query);
        let resp = self.client.get(&url).send().await?;
        let body = resp.text().await?;
        let document = Html::parse_document(&body);

        let result_selector = Selector::parse(".web-result").unwrap();
        let result_title_selector = Selector::parse(".result__a").unwrap();
        let result_url_selector = Selector::parse(".result__url").unwrap();
        let result_snippet_selector = Selector::parse(".result__snippet").unwrap();

        let results = document
            .select(&result_selector)
            .map(|result| {
                let title = result
                    .select(&result_title_selector)
                    .next()
                    .unwrap()
                    .text()
                    .collect::<Vec<_>>()
                    .join("");
                let link = result
                    .select(&result_url_selector)
                    .next()
                    .unwrap()
                    .text()
                    .collect::<Vec<_>>()
                    .join("")
                    .trim()
                    .to_string();
                let snippet = result
                    .select(&result_snippet_selector)
                    .next()
                    .unwrap()
                    .text()
                    .collect::<Vec<_>>()
                    .join("");

                SearchResult {
                    title,
                    link,
                    //url: String::from(url.value().attr("href").unwrap()),
                    snippet,
                }
            })
            .collect::<Vec<_>>();

        Ok(results)
    }
}

#[async_trait]
impl Tool for DDGSearcher {
    fn name(&self) -> String {
        "DDG Searcher".to_string()
    }

    fn description(&self) -> String {
        "Searches the web using DuckDuckGo's HTML interface.".to_string()
    }

    fn parameters(&self) -> Value {
        json!({
            "description": "This tool lets you search the web using DuckDuckGo. The input should be a search query.",
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query to send to DuckDuckGo"
                }
            },
            "required": ["query"]
        })
    }

    async fn call(&self, input: &str) -> Result<String, Box<dyn Error>> {
        let input_value = self.parse_input(input).await;
        self.run(input_value).await
    }

    async fn run(&self, input: Value) -> Result<String, Box<dyn Error>> {
        let query = input["query"].as_str().unwrap();
        let results = self.search(query).await?;
        let results_json = serde_json::to_string(&results)?;
        Ok(results_json)
    }

    async fn parse_input(&self, input: &str) -> Value {
        Tool::parse_input(self, input).await
    }
}
