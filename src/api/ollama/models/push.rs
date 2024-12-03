use serde::{Deserialize, Serialize};

use crate::api::ollama::Ollama;

impl Ollama {
    /// Upload a model to a model library. Requires registering for ollama.ai and adding a public key first.
    /// Push a model with a single response, only the final status will be returned.
    /// - `model_name` - The name of the model to push in the form of `<namespace>/<model>:<tag>`.
    /// - `allow_insecure` - Allow insecure connections to the library. Only use this if you are pushing to your library during development.
    pub async fn push_model(
        &self,
        model_name: String,
        allow_insecure: bool,
    ) -> crate::api::ollama::error::Result<PushModelStatus> {
        let request = PushModelRequest {
            model_name,
            allow_insecure,
            stream: false,
        };

        let url = format!("{}api/push", self.url_str());
        let serialized = serde_json::to_string(&request).map_err(|e| e.to_string())?;
        let res = self
            .reqwest_client
            .post(url)
            .body(serialized)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            return Err(res.text().await.unwrap_or_else(|e| e.to_string()).into());
        }

        let res = res.bytes().await.map_err(|e| e.to_string())?;
        let res = serde_json::from_slice::<PushModelStatus>(&res).map_err(|e| e.to_string())?;

        Ok(res)
    }
}

/// A push model request to Ollama.
#[derive(Debug, Clone, Serialize)]
struct PushModelRequest {
    #[serde(rename = "name")]
    model_name: String,
    #[serde(rename = "insecure")]
    allow_insecure: bool,
    stream: bool,
}

/// A push model status response from Ollama.
#[derive(Debug, Clone, Deserialize)]
pub struct PushModelStatus {
    #[serde(rename = "status")]
    pub message: String,
    pub digest: Option<String>,
    pub total: Option<u64>,
}
