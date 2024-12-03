pub mod copy;
pub mod create;
pub mod delete;
pub mod list_local;
pub mod pull;
pub mod push;
pub mod show_info;

use serde::{Deserialize, Serialize};

/// A local model pulled from Ollama.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalModel {
    pub name: String,
    pub modified_at: String,
    pub size: u64,
}

/// A model's info. Some fields may be empty if the model does not have them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    #[serde(default = "String::new")]
    pub license: String,
    #[serde(default = "String::new")]
    pub modelfile: String,
    #[serde(default = "String::new")]
    pub parameters: String,
    #[serde(default = "String::new")]
    pub template: String,
}
