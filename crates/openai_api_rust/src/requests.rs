use crate::mpart::Mpart as Multipart;

use crate::openai::OpenAI;
use crate::*;

#[cfg(not(test))]
use log::error;
use log::trace;

#[cfg(test)]
use std::eprintln as error;

pub trait Requests {
	fn post(&self, sub_url: &str, body: Json) -> ApiResult<Json>;
	fn post_multipart(&self, sub_url: &str, multipart: Multipart) -> ApiResult<Json>;
	fn get(&self, sub_url: &str) -> ApiResult<Json>;
}

impl Requests for OpenAI {
	fn post(&self, sub_url: &str, body: Json) -> ApiResult<Json> {
		trace!("===> 🚀\n\tPost api: {sub_url}, body: {body}");

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
		trace!("===> 🚀\n\tGet api: {sub_url}");

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
		trace!("===> 🚀\n\tPost multipart api: {sub_url}, multipart: {:?}", multipart);

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
			trace!("<== ✔️\n\tDone api: {sub_url}, resp: {json}");
			Ok(json)
		},
		Err(err) => match err {
			ureq::Error::Status(status, response) => {
				let raw = response.status_text().to_string();
				let error_msg = match response.into_json::<Json>() {
					Ok(json) => json.to_string(),
					Err(_) => raw,
				};
				error!("api: {sub_url}, status: {status}, error: {error_msg}");
				Err(Error::ApiError(format!("{error_msg}")))
			},
			ureq::Error::Transport(e) => {
				error!("api: {sub_url}, error: {:?}", e.to_string());
				Err(Error::RequestError(e.to_string()))
			},
		},
	}
}
