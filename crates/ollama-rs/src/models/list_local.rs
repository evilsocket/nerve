use serde::Deserialize;

use crate::Ollama;

use super::LocalModel;

impl Ollama {
    pub async fn list_local_models(&self) -> crate::error::Result<Vec<LocalModel>> {
        let url = format!("{}api/tags", self.url_str());
        let res = self
            .reqwest_client
            .get(url)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            return Err(res.text().await.unwrap_or_else(|e| e.to_string()).into());
        }

        let res = res.bytes().await.map_err(|e| e.to_string())?;
        let res =
            serde_json::from_slice::<ListLocalModelsResponse>(&res).map_err(|e| e.to_string())?;

        Ok(res.models)
    }
}

/// A response from Ollama containing a list of local models.
#[derive(Debug, Clone, Deserialize)]
struct ListLocalModelsResponse {
    models: Vec<LocalModel>,
}
