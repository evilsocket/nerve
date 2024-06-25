use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::generator::Client;

mod metrics;
pub(crate) mod naive;

// TODO: move to mini-rag crate.
pub type Embeddings = Vec<f64>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Configuration {
    pub path: String,
}

#[derive(Clone, Debug)]
pub struct Document {
    pub name: String,
    pub data: String,
}

#[async_trait]
pub trait VectorStore: Send {
    #[allow(clippy::borrowed_box)]
    fn new_with_generator(generator: Box<dyn Client>) -> Result<Self>
    where
        Self: Sized;

    async fn add(&mut self, document: Document) -> Result<()>;
    async fn retrieve(&self, query: &str, top_k: usize) -> Result<Vec<(Document, f64)>>;
}
