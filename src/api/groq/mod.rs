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

pub mod completion;
