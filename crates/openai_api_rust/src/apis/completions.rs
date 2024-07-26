// Given a prompt, the model will return one or more predicted completions,
// and can also return the probabilities of alternative tokens at each position.
// See: https://platform.openai.com/docs/api-reference/completions

//! Completions API

use std::collections::HashMap;

use crate::requests::Requests;
use crate::*;
use serde::{Deserialize, Serialize};

use super::{Usage, COMPLETION_CREATE};

/// Given a prompt, the model will return one or more predicted completions,
/// and can also return the probabilities of alternative tokens at each position.
#[derive(Debug, Serialize, Deserialize)]
pub struct Completion {
	pub id: Option<String>,
	pub object: Option<String>,
	pub created: Option<u64>,
	pub model: Option<String>,
	pub choices: Vec<Choice>,
	pub usage: Usage,
}

/// Request body for `Create completion` API
#[derive(Debug, Serialize, Deserialize)]
pub struct CompletionsBody {
	/// ID of the model to use
	pub model: String,
	/// The prompt(s) to generate completions for,
	/// encoded as a string, array of strings, array of tokens, or array of token arrays.
	/// Defaults to <|endoftext|>
	#[serde(skip_serializing_if = "Option::is_none")]
	pub prompt: Option<Vec<String>>,
	/// The suffix that comes after a completion of inserted text.
	/// Defaults to null
	#[serde(skip_serializing_if = "Option::is_none")]
	pub suffix: Option<String>,
	/// The maximum number of tokens to generate in the completion.
	/// The token count of your prompt plus max_tokens cannot exceed the model's context length.
	/// Most models have a context length of 2048 tokens (except for the newest models, which support 4096).
	/// Defaults to 16
	#[serde(skip_serializing_if = "Option::is_none")]
	pub max_tokens: Option<i32>,
	/// What sampling temperature to use, between 0 and 2.
	/// Higher values like 0.8 will make the output more random,
	/// while lower values like 0.2 will make it more focused and deterministic.
	/// We generally recommend altering this or top_p but not both.
	/// Defaults to 1
	#[serde(skip_serializing_if = "Option::is_none")]
	pub temperature: Option<f32>,
	/// An alternative to sampling with temperature, called nucleus sampling,
	/// where the model considers the results of the tokens with top_p probability mass.
	/// So 0.1 means only the tokens comprising the top 10% probability mass are considered.
	/// We generally recommend altering this or temperature but not both.
	/// Defaults to 1
	#[serde(skip_serializing_if = "Option::is_none")]
	pub top_p: Option<f32>,
	/// How many completions to generate for each prompt.
	/// Note: Because this parameter generates many completions,
	/// it can quickly consume your token quota.
	/// Use carefully and ensure that you have reasonable settings for max_tokens and stop.
	/// Defaults to 1
	#[serde(skip_serializing_if = "Option::is_none")]
	pub n: Option<i32>,
	/// Whether to stream back partial progress.
	/// If set, tokens will be sent as data-only server-sent events as they become available,
	/// with the stream terminated by a data: [DONE] message.
	/// Defaults to false
	#[serde(skip_serializing_if = "Option::is_none")]
	pub stream: Option<bool>,
	/// Include the log probabilities on the logprobs most likely tokens,
	/// as well the chosen tokens. For example, if logprobs is 5,
	/// the API will return a list of the 5 most likely tokens.
	/// The API will always return the logprob of the sampled token,
	/// so there may be up to logprobs+1 elements in the response.
	/// The maximum value for logprobs is 5. If you need more than this,
	/// please contact us through our Help center and describe your use case.
	/// Defaults to null
	#[serde(skip_serializing_if = "Option::is_none")]
	pub logprobs: Option<i32>,
	/// Echo back the prompt in addition to the completion
	/// Defaults to false
	#[serde(skip_serializing_if = "Option::is_none")]
	pub echo: Option<bool>,
	/// Up to 4 sequences where the API will stop generating further tokens.
	/// The returned text will not contain the stop sequence.
	/// Defaults to null
	#[serde(skip_serializing_if = "Option::is_none")]
	pub stop: Option<Vec<String>>,
	/// Number between -2.0 and 2.0.
	/// Positive values penalize new tokens based on whether they appear in the text so far,
	/// increasing the model's likelihood to talk about new topics.
	/// See more: https://platform.openai.com/docs/api-reference/parameter-details
	/// Defaults to 0
	#[serde(skip_serializing_if = "Option::is_none")]
	pub presence_penalty: Option<f32>,
	/// Number between -2.0 and 2.0.
	/// Positive values penalize new tokens based on their existing frequency in the text so far,
	/// decreasing the model's likelihood to repeat the same line verbatim.
	/// Defaults to 0
	#[serde(skip_serializing_if = "Option::is_none")]
	pub frequency_penalty: Option<f32>,
	/// Generates best_of completions server-side and returns
	/// the "best" (the one with the highest log probability per token). Results cannot be streamed.
	/// When used with n, best_of controls the number of candidate completions
	/// and n specifies how many to return – best_of must be greater than n.
	/// Note: Because this parameter generates many completions,
	/// it can quickly consume your token quota.
	/// Use carefully and ensure that you have reasonable settings for max_tokens and stop.
	/// Defaults to 1
	#[serde(skip_serializing_if = "Option::is_none")]
	pub best_of: Option<i32>,
	/// Modify the likelihood of specified tokens appearing in the completion.
	/// Accepts a json object that maps tokens (specified by their token ID in the GPT tokenizer)
	/// to an associated bias value from -100 to 100. You can use this tokenizer tool (which works for both GPT-2 and GPT-3) to convert text to token IDs. Mathematically, the bias is added to the logits generated by the model prior to sampling. The exact effect will vary per model, but values between -1 and 1 should decrease or increase likelihood of selection; values like -100 or 100 should result in a ban or exclusive selection of the relevant token.
	/// As an example, you can pass {"50256": -100} to prevent the <|endoftext|> token from being generated.
	/// Defaults to null
	#[serde(skip_serializing_if = "Option::is_none")]
	pub logit_bias: Option<HashMap<String, String>>,
	/// A unique identifier representing your end-user, which can help OpenAI to monitor and detect abuse.
	/// Learn more: https://platform.openai.com/docs/guides/safety-best-practices/end-user-ids
	#[serde(skip_serializing_if = "Option::is_none")]
	pub user: Option<String>,
}

pub trait CompletionsApi {
	/// Creates a completion for the provided prompt and parameters
	fn completion_create(&self, completions_body: &CompletionsBody) -> ApiResult<Completion>;
}

impl CompletionsApi for OpenAI {
	fn completion_create(&self, completions_body: &CompletionsBody) -> ApiResult<Completion> {
		let request_body = serde_json::to_value(completions_body).unwrap();
		let res = self.post(COMPLETION_CREATE, request_body)?;
		let completion: Completion = serde_json::from_value(res.clone()).unwrap();
		Ok(completion)
	}
}

#[cfg(test)]
mod tests {
	use crate::openai::new_test_openai;

	use super::{CompletionsApi, CompletionsBody};

	#[test]
	fn test_completions() {
		let openai = new_test_openai();
		let body = CompletionsBody {
			model: "babbage-002".to_string(),
			prompt: Some(vec!["Say this is a test".to_string()]),
			suffix: None,
			max_tokens: Some(7),
			temperature: Some(0_f32),
			top_p: Some(0_f32),
			n: Some(2),
			stream: Some(false),
			logprobs: None,
			echo: None,
			stop: Some(vec!["\n".to_string()]),
			presence_penalty: None,
			frequency_penalty: None,
			best_of: None,
			logit_bias: None,
			user: None,
		};
		let rs = openai.completion_create(&body);
		let choice = rs.unwrap().choices;
		let text = &choice[0].text.as_ref().unwrap();
		assert!(text.contains("this"));
	}
}
