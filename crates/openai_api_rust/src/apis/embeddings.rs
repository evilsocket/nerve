// Get a vector representation of a given input
// that can be easily consumed by machine learning models and algorithms.
// See: https://platform.openai.com/docs/api-reference/embeddings

//! Embeddings API

use serde::{Deserialize, Serialize};

use crate::requests::Requests;
use crate::*;

use super::{Usage, EMBEDDINGS_CREATE};

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingsBody {
	/// ID of the model to use. You can use the List models API to see all of your available models,
	/// or see our Model overview for descriptions of them.
	pub model: String,
	/// Input text to get embeddings for, encoded as a string or array of tokens. To get embeddings for multiple inputs in a single request,
	/// pass an array of strings or array of token arrays. Each input must not exceed 8192 tokens in length.
	pub input: Vec<String>,
	/// A unique identifier representing your end-user, which can help OpenAI to monitor and detect abuse.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub user: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Embeddings {
	pub object: Option<String>,
	pub data: Option<Vec<EmbeddingData>>,
	pub model: String,
	pub usage: Usage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingData {
	pub object: Option<String>,
	pub embedding: Option<Vec<f64>>,
	pub index: i32,
}

pub trait EmbeddingsApi {
	/// Creates an embedding vector representing the input text.
	fn embeddings_create(&self, embeddings_body: &EmbeddingsBody) -> ApiResult<Embeddings>;
}

impl EmbeddingsApi for OpenAI {
	fn embeddings_create(&self, embeddings_body: &EmbeddingsBody) -> ApiResult<Embeddings> {
		let request_body = serde_json::to_value(embeddings_body).unwrap();
		let res = self.post(EMBEDDINGS_CREATE, request_body)?;
		let embeddings: Embeddings = serde_json::from_value(res.clone()).unwrap();
		Ok(embeddings)
	}
}

#[cfg(test)]
mod tests {
	use crate::{
		apis::embeddings::{EmbeddingsApi, EmbeddingsBody},
		openai::new_test_openai,
	};

	#[test]
	fn test_embedding_create() {
		let openai = new_test_openai();
		let body = EmbeddingsBody {
			model: "text-embedding-ada-002".to_string(),
			input: vec!["The food was delicious and the waiter...".to_string()],
			user: None,
		};
		let rs = openai.embeddings_create(&body);
		let embeddings = rs.unwrap().data;
		let embedding = embeddings.as_ref().unwrap().get(0).unwrap();
		let f = embedding.embedding.as_ref().unwrap();
		assert!(!f.is_empty());
	}
}
