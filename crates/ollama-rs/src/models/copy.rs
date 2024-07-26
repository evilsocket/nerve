use serde::Serialize;

use crate::Ollama;

impl Ollama {
    /// Copy a model. Creates a model with another name from an existing model.
    pub async fn copy_model(
        &self,
        source: String,
        destination: String,
    ) -> crate::error::Result<()> {
        let request = CopyModelRequest {
            source,
            destination,
        };

        let url = format!("{}api/copy", self.url_str());
        let serialized = serde_json::to_string(&request).map_err(|e| e.to_string())?;
        let res = self
            .reqwest_client
            .post(url)
            .body(serialized)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if res.status().is_success() {
            Ok(())
        } else {
            Err(res.text().await.unwrap_or_else(|e| e.to_string()).into())
        }
    }
}

/// A copy model request to Ollama.
#[derive(Serialize)]
struct CopyModelRequest {
    source: String,
    destination: String,
}
