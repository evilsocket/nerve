//! Provides a simple client implementation for the [groq cloud API](https://console.groq.com/playground).
//! You can learn more about the API provided [API Documentation](https://console.groq.com/docs/quickstart)
//! This crate uses [`reqwest`], [`reqwest_eventsource`], [`tokio`], [`serde`], [`serde_json`], [`anyhow`],
//! [`chrono`],[`futures`]
//!
//! # MSRV
//! 1.78.0
//!
//! # Usage
//! ```sh
//! cargo add groq-api-rs
//! ```
//!
//! # Example
//! Request a completion object from Groq
//! ```
//! use groq_api_rs::completion::{client::Groq, message::Message, request::builder};
//!
//! async fn create_completion() -> anyhow::Result<()> {
//!     let messages = vec![Message::UserMessage {
//!         role: Some("user".to_string()),
//!         content: Some("Explain the importance of fast language models".to_string()),
//!         name: None,
//!         tool_call_id: None,
//!     }];
//!     let request = builder::RequestBuilder::new("mixtral-8x7b-32768".to_string());
//!     let api_key = env!("GROQ_API_KEY");
//!
//!     let mut client = Groq::new(api_key);
//!     client.add_messages(messages);
//!
//!     let res = client.create(request).await;
//!     assert!(res.is_ok());
//!     Ok(())
//! }
//! ```
//!
//! Request a completion chunk object from Groq using stream option implemented with SSE
//! ```
//! use groq_api_rs::completion::{client::Groq, message::Message, request::builder};
//! async fn create_stream_completion() -> anyhow::Result<()> {
//!     let messages = vec![Message::UserMessage {
//!         role: Some("user".to_string()),
//!         content: Some("Explain the importance of fast language models".to_string()),
//!         name: None,
//!         tool_call_id: None,
//!     }];
//!     let request =
//!         builder::RequestBuilder::new("mixtral-8x7b-32768".to_string()).with_stream(true);
//!     let api_key = env!("GROQ_API_KEY");
//!
//!     let mut client = Groq::new(api_key);
//!     client.add_messages(messages);
//!
//!     let res = client.create(request).await;
//!     assert!(res.is_ok());
//!     Ok(())
//! }
//! ```
//!
//! Example that the completion can return Error Object and augmented with HTTP status code.
//! ```
//! use groq_api_rs::completion::{client::Groq, message::Message, request::builder};
//! async fn error_does_return() -> anyhow::Result<()> {
//!     let messages = vec![Message::UserMessage {
//!         role: Some("user".to_string()),
//!         content: Some("Explain the importance of fast language models".to_string()),
//!         name: None,
//!         tool_call_id: None,
//!     }];
//!     let request =
//!         builder::RequestBuilder::new("mixtral-8x7b-32768".to_string()).with_stream(true);
//!     let api_key = "";
//!
//!     let mut client = Groq::new(api_key);
//!     client.add_messages(messages);
//!
//!     let res = client.create(request).await;
//!     assert!(res.is_err());
//!     eprintln!("{}", res.unwrap_err());
//!     Ok(())
//! }
//! ```

pub mod completion;
