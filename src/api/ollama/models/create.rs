use serde::{Deserialize, Serialize};

use crate::api::ollama::Ollama;

impl Ollama {
    /// Create a model with a single response, only the final status will be returned.
    pub async fn create_model(
        &self,
        request: CreateModelRequest,
    ) -> crate::api::ollama::error::Result<CreateModelStatus> {
        let url = format!("{}api/create", self.url_str());
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
        let res = serde_json::from_slice::<CreateModelStatus>(&res).map_err(|e| e.to_string())?;

        Ok(res)
    }
}

/// A create model request to Ollama.
#[derive(Serialize)]
pub struct CreateModelRequest {
    #[serde(rename = "name")]
    model_name: String,
    path: Option<String>,
    modelfile: Option<String>,
    stream: bool,
}

impl CreateModelRequest {
    /// Create a model described in the Modelfile at `path`.
    pub fn path(model_name: String, path: String) -> Self {
        Self {
            model_name,
            path: Some(path),
            modelfile: None,
            stream: false,
        }
    }

    /// Create a model described by the Modelfile contents passed to `modelfile`.
    pub fn modelfile(model_name: String, modelfile: String) -> Self {
        Self {
            model_name,
            path: None,
            modelfile: Some(modelfile),
            stream: false,
        }
    }
}

/// A create model status response from Ollama.
#[derive(Deserialize, Debug)]
pub struct CreateModelStatus {
    #[serde(rename = "status")]
    pub message: String,
}
