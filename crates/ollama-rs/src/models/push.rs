use serde::{Deserialize, Serialize};

use crate::Ollama;

/// A stream of `PushModelStatus` objects.
#[cfg(feature = "stream")]
pub type PushModelStatusStream = std::pin::Pin<
    Box<dyn tokio_stream::Stream<Item = crate::error::Result<PushModelStatus>> + Send>,
>;

impl Ollama {
    #[cfg(feature = "stream")]
    /// Upload a model to a model library. Requires registering for ollama.ai and adding a public key first.
    /// Push a model with streaming, meaning that each new status will be streamed.
    /// - `model_name` - The name of the model to push in the form of `<namespace>/<model>:<tag>`.
    /// - `allow_insecure` - Allow insecure connections to the library. Only use this if you are pushing to your library during development.
    pub async fn push_model_stream(
        &self,
        model_name: String,
        allow_insecure: bool,
    ) -> crate::error::Result<PushModelStatusStream> {
        use crate::error::OllamaError;
        use tokio_stream::StreamExt;

        let request = PushModelRequest {
            model_name,
            allow_insecure,
            stream: true,
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

        let stream = Box::new(res.bytes_stream().map(|res| match res {
            Ok(bytes) => {
                let res = serde_json::from_slice::<PushModelStatus>(&bytes);
                match res {
                    Ok(res) => Ok(res),
                    Err(e) => {
                        let err = serde_json::from_slice::<crate::error::OllamaError>(&bytes);
                        match err {
                            Ok(err) => Err(err),
                            Err(_) => Err(OllamaError::from(format!(
                                "Failed to deserialize response: {}",
                                e
                            ))),
                        }
                    }
                }
            }
            Err(e) => Err(OllamaError::from(format!("Failed to read response: {}", e))),
        }));

        Ok(std::pin::Pin::from(stream))
    }

    /// Upload a model to a model library. Requires registering for ollama.ai and adding a public key first.
    /// Push a model with a single response, only the final status will be returned.
    /// - `model_name` - The name of the model to push in the form of `<namespace>/<model>:<tag>`.
    /// - `allow_insecure` - Allow insecure connections to the library. Only use this if you are pushing to your library during development.
    pub async fn push_model(
        &self,
        model_name: String,
        allow_insecure: bool,
    ) -> crate::error::Result<PushModelStatus> {
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
