use serde::{Deserialize, Serialize};

use crate::api::ollama::Ollama;

impl Ollama {
    /// Pull a model with a single response, only the final status will be returned.
    /// - `model_name` - The name of the model to pull.
    /// - `allow_insecure` - Allow insecure connections to the library. Only use this if you are pulling from your own library during development.
    pub async fn pull_model(
        &self,
        model_name: String,
        allow_insecure: bool,
    ) -> crate::api::ollama::error::Result<PullModelStatus> {
        let request = PullModelRequest {
            model_name,
            allow_insecure,
            stream: false,
        };

        let url = format!("{}api/pull", self.url_str());
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
        let res = serde_json::from_slice::<PullModelStatus>(&res).map_err(|e| e.to_string())?;

        Ok(res)
    }
}

/// A pull model request to Ollama.
#[derive(Debug, Clone, Serialize)]
struct PullModelRequest {
    #[serde(rename = "name")]
    model_name: String,
    #[serde(rename = "insecure")]
    allow_insecure: bool,
    stream: bool,
}

/// A pull model status response from Ollama.
#[derive(Debug, Clone, Deserialize)]
pub struct PullModelStatus {
    #[serde(rename = "status")]
    pub message: String,
    pub digest: Option<String>,
    pub total: Option<u64>,
    pub completed: Option<u64>,
}
