use crate::mpart::Mpart as Multipart;

use crate::openai::OpenAI;
use crate::*;

#[cfg(not(test))]
use log::{debug, error, info};

#[cfg(test)]
use std::{eprintln as error, println as info, println as debug};

pub trait Requests {
	fn post(&self, sub_url: &str, body: Json) -> ApiResult<Json>;
	fn post_multipart(&self, sub_url: &str, multipart: Multipart) -> ApiResult<Json>;
	fn get(&self, sub_url: &str) -> ApiResult<Json>;
}

impl Requests for OpenAI {
	fn post(&self, sub_url: &str, body: Json) -> ApiResult<Json> {
		info!("===> 🚀\n\tPost api: {sub_url}, body: {body}");

		let response = self
			.agent
			.post(&(self.api_url.clone() + sub_url))
			.set("Content-Type", "application/json")
			.set("OpenAI-Organization", &self.auth.organization.clone().unwrap_or_default())
			.set("Authorization", &format!("Bearer {}", self.auth.api_key))
			.send_json(body);

		deal_response(response, sub_url)
	}

	fn get(&self, sub_url: &str) -> ApiResult<Json> {
		info!("===> 🚀\n\tGet api: {sub_url}");

		let response = self
			.agent
			.get(&(self.api_url.clone() + sub_url))
			.set("Content-Type", "application/json")
			.set("OpenAI-Organization", &self.auth.organization.clone().unwrap_or_default())
			.set("Authorization", &format!("Bearer {}", self.auth.api_key))
			.call();

		deal_response(response, sub_url)
	}

	fn post_multipart(&self, sub_url: &str, mut multipart: Multipart) -> ApiResult<Json> {
		info!("===> 🚀\n\tPost multipart api: {sub_url}, multipart: {:?}", multipart);

		let form_data = multipart.prepare().unwrap();

		let response = self
			.agent
			.post(&(self.api_url.clone() + sub_url))
			.set("Content-Type", &format!("multipart/form-data; boundary={}", form_data.boundary()))
			.set("OpenAI-Organization", &self.auth.organization.clone().unwrap_or_default())
			.set("Authorization", &format!("Bearer {}", self.auth.api_key))
			.send(form_data);

		deal_response(response, sub_url)
	}
}

fn deal_response(response: Result<ureq::Response, ureq::Error>, sub_url: &str) -> ApiResult<Json> {
	match response {
		Ok(resp) => {
			let json = resp.into_json::<Json>().unwrap();
			debug!("<== ✔️\n\tDone api: {sub_url}, resp: {json}");
			return Ok(json);
		},
		Err(err) => match err {
			ureq::Error::Status(status, response) => {
				let error_msg = response.into_json::<Json>().unwrap();
				error!("<== ❌\n\tError api: {sub_url}, status: {status}, error: {error_msg}");
				return Err(Error::ApiError(format!("{error_msg}")));
			},
			ureq::Error::Transport(e) => {
				error!("<== ❌\n\tError api: {sub_url}, error: {:?}", e.to_string());
				return Err(Error::RequestError(e.to_string()));
			},
		},
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::openai;
	use ureq::json;

	#[test]
	fn test_post() {
		let openai = openai::new_test_openai();
		let body = json!({
			"model": "gpt-3.5-turbo",
			"messages": [{"role": "user", "content": "Say this is a test!"}],
			"temperature": 0.7
		});
		let sub_url = "chat/completions";
		let result = openai.post(sub_url, body).unwrap();
		assert!(result.to_string().contains("This is a test"));
	}

	#[test]
	fn test_get() {
		let openai = openai::new_test_openai();
		let resp = openai.get("models").unwrap();
		assert!(resp.to_string().contains("babbage-002"));
	}
}
