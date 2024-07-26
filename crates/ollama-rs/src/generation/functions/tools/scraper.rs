use crate::generation::functions::tools::Tool;
use async_trait::async_trait;
use reqwest::Client;
use scraper::{Html, Selector};
use serde_json::{json, Value};
use std::error::Error;

pub struct Scraper {}

impl Default for Scraper {
    fn default() -> Self {
        Self::new()
    }
}

impl Scraper {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Tool for Scraper {
    fn name(&self) -> String {
        "Website Scraper".to_string()
    }

    fn description(&self) -> String {
        "Scrapes text content from websites and splits it into manageable chunks.".to_string()
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "website": {
                    "type": "string",
                    "description": "The URL of the website to scrape"
                }
            },
            "required": ["website"]
        })
    }

    async fn run(&self, input: Value) -> Result<String, Box<dyn Error>> {
        let website = input["website"].as_str().ok_or("Website URL is required")?;
        let client = Client::new();
        let response = client.get(website).send().await?.text().await?;

        let document = Html::parse_document(&response);
        let selector = Selector::parse("p, h1, h2, h3, h4, h5, h6").unwrap();
        let elements: Vec<String> = document
            .select(&selector)
            .map(|el| el.text().collect::<Vec<_>>().join(" "))
            .collect();
        let body = elements.join(" ");

        let sentences: Vec<String> = body.split(". ").map(|s| s.to_string()).collect();
        let formatted_content = sentences.join("\n\n");

        Ok(formatted_content)
    }
}
