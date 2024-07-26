#![cfg(feature = "function-calling")]

use ollama_rs::{
    generation::chat::ChatMessage,
    generation::functions::tools::{DDGSearcher, Scraper, StockScraper},
    generation::functions::{FunctionCallRequest, NousFunctionCall},
    Ollama,
};
use std::sync::Arc;

#[tokio::test]
async fn test_send_function_call() {
    /// Model to be used, make sure it is tailored towards "function calling", such as:
    /// - OpenAIFunctionCall: not model specific, degraded performance
    /// - NousFunctionCall: adrienbrault/nous-hermes2pro:Q8_0
    const MODEL: &str = "adrienbrault/nous-hermes2pro:Q8_0";

    const PROMPT: &str = "Aside from the Apple Remote, what other device can control the program Apple Remote was originally designed to interact with?";
    let user_message = ChatMessage::user(PROMPT.to_string());

    let scraper_tool = Arc::new(Scraper::new());
    let ddg_search_tool = Arc::new(DDGSearcher::new());
    let parser = Arc::new(NousFunctionCall::new());

    let ollama = Ollama::default();
    let result = ollama
        .send_function_call(
            FunctionCallRequest::new(
                MODEL.to_string(),
                vec![scraper_tool, ddg_search_tool],
                vec![user_message],
            ),
            parser,
        )
        .await
        .unwrap();

    assert!(result.done);
}

#[tokio::test]
async fn test_send_function_call_with_history() {
    /// Model to be used, make sure it is tailored towards "function calling", such as:
    /// - OpenAIFunctionCall: not model specific, degraded performance
    /// - NousFunctionCall: adrienbrault/nous-hermes2pro:Q8_0
    const MODEL: &str = "adrienbrault/nous-hermes2pro:Q8_0";

    const PROMPT: &str = "Aside from the Apple Remote, what other device can control the program Apple Remote was originally designed to interact with?";
    let user_message = ChatMessage::user(PROMPT.to_string());

    let scraper_tool = Arc::new(Scraper::new());
    let ddg_search_tool = Arc::new(DDGSearcher::new());
    let parser = Arc::new(NousFunctionCall::new());

    let mut ollama = Ollama::new_default_with_history(30);
    let result = ollama
        .send_function_call_with_history(
            FunctionCallRequest::new(
                MODEL.to_string(),
                vec![scraper_tool, ddg_search_tool],
                vec![user_message],
            ),
            parser,
            "default".to_string(),
        )
        .await
        .unwrap();

    assert!(result.done);
}

#[tokio::test]
async fn test_send_function_call_finance() {
    /// Model to be used, make sure it is tailored towards "function calling", such as:
    /// - OpenAIFunctionCall: not model specific, degraded performance
    /// - NousFunctionCall: adrienbrault/nous-hermes2pro:Q8_0
    const MODEL: &str = "adrienbrault/nous-hermes2pro:Q8_0";

    const PROMPT: &str = "What are the current risk factors to $APPL?";
    let user_message = ChatMessage::user(PROMPT.to_string());

    let stock_scraper = Arc::new(StockScraper::new());
    let parser = Arc::new(NousFunctionCall::new());

    let ollama = Ollama::default();
    let result = ollama
        .send_function_call(
            FunctionCallRequest::new(MODEL.to_string(), vec![stock_scraper], vec![user_message]),
            parser,
        )
        .await
        .unwrap();

    assert!(result.done);
}
