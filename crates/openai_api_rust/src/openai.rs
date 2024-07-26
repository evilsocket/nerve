use serde::{Deserialize, Serialize};
use ureq::{Agent, AgentBuilder};

#[derive(Debug, Serialize, Deserialize)]
pub struct Auth {
	pub api_key: String,
	pub organization: Option<String>,
}

impl Clone for Auth {
	fn clone(&self) -> Self {
		Self { api_key: self.api_key.clone(), organization: self.organization.clone() }
	}
}

#[allow(dead_code)]
impl Auth {
	pub fn new(api_key: &str) -> Auth {
		Auth { api_key: api_key.to_string(), organization: None }
	}

	pub fn from_env() -> Result<Self, String> {
		let api_key =
			std::env::var("OPENAI_API_KEY").map_err(|_| "Missing OPENAI_API_KEY".to_string())?;
		Ok(Self { api_key, organization: None })
	}
}

#[derive(Debug)]
pub struct OpenAI {
	pub auth: Auth,
	pub api_url: String,
	pub(crate) agent: Agent,
}

impl Clone for OpenAI {
	fn clone(&self) -> Self {
		Self { auth: self.auth.clone(), api_url: self.api_url.clone(), agent: self.agent.clone() }
	}
}

#[allow(dead_code)]
impl OpenAI {
	pub fn new(auth: Auth, api_url: &str) -> OpenAI {
		OpenAI { auth, api_url: api_url.to_string(), agent: AgentBuilder::new().build() }
	}

	pub fn set_proxy(mut self, proxy: &str) -> OpenAI {
		let proxy = ureq::Proxy::new(proxy).unwrap();
		self.agent = ureq::AgentBuilder::new().proxy(proxy).build();
		self
	}

	pub fn use_env_proxy(mut self) -> OpenAI {
		let proxy = match (std::env::var("http_proxy"), std::env::var("https_proxy")) {
			(Ok(http_proxy), _) => Some(http_proxy),
			(_, Ok(https_proxy)) => Some(https_proxy),
			_ => {
				log::warn!("Missing http_proxy or https_proxy");
				None
			},
		};
		if let Some(proxy) = proxy {
			let proxy = ureq::Proxy::new(&proxy).unwrap();
			self.agent = ureq::AgentBuilder::new().proxy(proxy).build();
		}
		self
	}
}

#[cfg(test)]
pub fn new_test_openai() -> OpenAI {
	let auth = Auth::from_env().unwrap();
	OpenAI::new(auth, "https://api.openai.com/v1/").use_env_proxy()
}
