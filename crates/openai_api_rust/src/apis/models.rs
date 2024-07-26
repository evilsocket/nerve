// List and describe the various models available in the API.
// You can refer to the Models documentation to understand
// what models are available and the differences between them.
// See: https://platform.openai.com/docs/api-reference/models

//! Models API
use crate::requests::Requests;
use crate::*;
use serde::{Deserialize, Serialize};

use super::MODELS_LIST;
use super::MODELS_RETRIEVE;

/// List and describe the various models available in the API.
/// You can refer to the [Models](https://platform.openai.com/docs/models) documentation
/// to understand what models are available and the differences between them.
#[derive(Debug, Serialize, Deserialize)]
pub struct Model {
	pub id: String,
	pub object: Option<String>,
	pub owned_by: Option<String>,
}

pub trait ModelsApi {
	/// Lists the currently available models,
	/// and provides basic information about each one such as the owner and availability.
	fn models_list(&self) -> ApiResult<Vec<Model>>;
	/// Retrieves a model instance,
	/// providing basic information about the model such as the owner and permissioning.
	fn models_retrieve(&self, model_id: &str) -> ApiResult<Model>;
}

impl ModelsApi for OpenAI {
	fn models_list(&self) -> ApiResult<Vec<Model>> {
		let res: Json = self.get(MODELS_LIST)?;
		let data = res.as_object().unwrap().get("data");
		if let Some(data) = data {
			let models: Vec<Model> = serde_json::from_value(data.clone()).unwrap();
			return Ok(models);
		}
		Err(Error::ApiError("No data".to_string()))
	}

	fn models_retrieve(&self, model_id: &str) -> ApiResult<Model> {
		let res: Json = self.get(&(MODELS_RETRIEVE.to_owned() + model_id))?;
		let model: Model = serde_json::from_value(res).unwrap();
		Ok(model)
	}
}

#[cfg(test)]
mod tests {
	use crate::{apis::models::ModelsApi, openai::new_test_openai};

	#[test]
	fn test_models() {
		let openai = new_test_openai();
		let models = openai.models_list().unwrap();
		assert!(!models.is_empty());
	}

	#[test]
	fn test_get_model() {
		let openai = new_test_openai();
		let model = openai.models_retrieve("babbage-002").unwrap();
		assert_eq!("babbage-002", model.id);
	}
}
