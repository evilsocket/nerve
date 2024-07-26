pub mod finance;
pub mod scraper;
pub mod search_ddg;

pub use self::finance::StockScraper;
pub use self::scraper::Scraper;
pub use self::search_ddg::DDGSearcher;

use async_trait::async_trait;
use serde_json::{json, Value};
use std::error::Error;
use std::string::String;

#[async_trait]
pub trait Tool: Send + Sync {
    /// Returns the name of the tool.
    fn name(&self) -> String;

    /// Provides a description of what the tool does and when to use it.
    fn description(&self) -> String;

    /// Returns the parameters for OpenAI-like function call.
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "input": {
                    "type": "string",
                    "description": self.description()
                }
            },
            "required": ["input"]
        })
    }

    /// Processes an input string and executes the tool's functionality, returning a `Result`.
    async fn call(&self, input: &str) -> Result<String, Box<dyn Error>> {
        let input = self.parse_input(input).await;
        self.run(input).await
    }

    /// Executes the core functionality of the tool.
    async fn run(&self, input: Value) -> Result<String, Box<dyn Error>>;

    /// Parses the input string.
    async fn parse_input(&self, input: &str) -> Value {
        log::info!("Using default implementation: {}", input);
        match serde_json::from_str::<Value>(input) {
            Ok(input) => {
                if input["input"].is_string() {
                    Value::String(input["input"].as_str().unwrap().to_string())
                } else {
                    Value::String(input.to_string())
                }
            }
            Err(_) => Value::String(input.to_string()),
        }
    }
}
