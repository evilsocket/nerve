//! Pull parser for [CommonMark](https://commonmark.org). This crate provides a [Parser](struct.Parser.html) struct
//! which is an iterator over [Event](enum.Event.html)s. This iterator can be used
//! directly, or to output HTML using the [HTML module](html/index.html).
//!
//! By default, only CommonMark features are enabled. To use extensions like tables,
//! footnotes or task lists, enable them by setting the corresponding flags in the
//! [Options](struct.Options.html) struct.
//!
//! # Example
//! ```rust
//! use openai_api_rust::*;
//! use openai_api_rust::chat::*;
//! use openai_api_rust::completions::*;
//!
//! fn main() {
//!     // Load API key from environment OPENAI_API_KEY.
//!     // You can also hadcode through `Auth::new(<your_api_key>)`, but it is not recommended.
//!     let auth = Auth::from_env().unwrap();
//!     let openai = OpenAI::new(auth, "https://api.openai.com/v1/");
//!     let body = ChatBody {
//!         model: "gpt-3.5-turbo".to_string(),
//!         max_tokens: Some(7),
//!         temperature: Some(0_f32),
//!         top_p: Some(0_f32),
//!         n: Some(2),
//!         stream: Some(false),
//!         stop: None,
//!         presence_penalty: None,
//!         frequency_penalty: None,
//!         logit_bias: None,
//!         user: None,
//!         messages: vec![Message { role: Role::User, content: "Hello!".to_string() }],
//!     };
//!     let rs = openai.chat_completion_create(&body);
//!     let choice = rs.unwrap().choices;
//!     let message = &choice[0].message.as_ref().unwrap();
//!     assert!(message.content.contains("Hello"));
//! }
//! ```
//!
//! ## Use proxy
//!
//! ```rust
//! // Load proxy from env
//! let openai = OpenAI::new(auth, "https://api.openai.com/v1/")
//!        .use_env_proxy();
//!
//! // Set the proxy manually
//! let openai = OpenAI::new(auth, "https://api.openai.com/v1/")
//!        .set_proxy("http://127.0.0.1:1080");
//! ```

#![warn(unused_crate_dependencies)]

pub mod apis;
use std::fmt::{self, Display, Formatter};

pub use apis::*;
pub mod openai;
pub use openai::*;
mod mpart;
mod requests;

use log as _;

pub type Json = serde_json::Value;
pub type ApiResult<T> = Result<T, Error>;

#[derive(Debug)]
pub enum Error {
	/// An Error returned by the API
	ApiError(String),
	/// An Error not related to the API
	RequestError(String),
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self {
			Error::ApiError(msg) => write!(f, "API error: {}", msg),
			Error::RequestError(msg) => write!(f, "Request error: {}", msg),
		}
	}
}
