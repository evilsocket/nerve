use crate::generation::functions::tools::Tool;
use async_trait::async_trait;
use reqwest::Client;
use scraper::{Html, Selector};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::error::Error;

pub struct StockScraper {
    base_url: String,
    language: String,
}

impl Default for StockScraper {
    fn default() -> Self {
        Self::new()
    }
}

impl StockScraper {
    pub fn new() -> Self {
        StockScraper {
            base_url: "https://www.google.com/finance".to_string(),
            language: "en".to_string(),
        }
    }

    // Changed to an async function
    pub async fn scrape(
        &self,
        exchange: &str,
        ticker: &str,
    ) -> Result<HashMap<String, String>, Box<dyn Error>> {
        let target_url = format!(
            "{}/quote/{}:{}?hl={}",
            self.base_url, ticker, exchange, self.language
        );
        let client = Client::new();
        let response = client.get(&target_url).send().await?; // Make the request asynchronously
        let content = response.text().await?; // Asynchronously get the text of the response
        let document = Html::parse_document(&content);

        let items_selector = Selector::parse("div.gyFHrc").unwrap();
        let desc_selector = Selector::parse("div.mfs7Fc").unwrap();
        let value_selector = Selector::parse("div.P6K39c").unwrap();

        let mut stock_description = HashMap::new();

        for item in document.select(&items_selector) {
            if let Some(item_description) = item.select(&desc_selector).next() {
                if let Some(item_value) = item.select(&value_selector).next() {
                    stock_description.insert(
                        item_description.text().collect::<Vec<_>>().join(""),
                        item_value.text().collect::<Vec<_>>().join(""),
                    );
                }
            }
        }

        Ok(stock_description)
    }
}

#[async_trait]
impl Tool for StockScraper {
    fn name(&self) -> String {
        "Stock Scraper".to_string()
    }

    fn description(&self) -> String {
        "Scrapes stock information from Google Finance.".to_string()
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "exchange": {
                    "type": "string",
                    "description": "The stock exchange market identifier code (MIC)"
                },
                "ticker": {
                    "type": "string",
                    "description": "The ticker symbol of the stock"
                }
            },
            "required": ["exchange", "ticker"]
        })
    }

    async fn run(&self, input: Value) -> Result<String, Box<dyn Error>> {
        let exchange = input["exchange"].as_str().ok_or("Exchange is required")?;
        let ticker = input["ticker"].as_str().ok_or("Ticker is required")?;

        let result = self.scrape(exchange, ticker).await?;
        Ok(serde_json::to_string(&result)?)
    }
}
