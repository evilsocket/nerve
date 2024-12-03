use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image(String);

impl Image {
    pub fn from_base64(base64: &str) -> Self {
        Self(base64.to_string())
    }
}
