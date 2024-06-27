#[macro_use]
extern crate anyhow;

use anyhow::Result;
use async_trait::async_trait;
use naive::NaiveVectorStore;
use serde::{Deserialize, Serialize};

pub(crate) use document::Document;

pub mod document;
pub mod metrics;
pub mod naive;

pub type Embeddings = Vec<f64>;

#[async_trait]
pub trait Embedder: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Embeddings>;
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Configuration {
    pub source_path: String,
    pub data_path: String,
    pub chunk_size: Option<usize>,
}

#[async_trait]
pub trait VectorStore: Send {
    #[allow(clippy::borrowed_box)]
    async fn new(embedder: Box<dyn Embedder>, config: Configuration) -> Result<Self>
    where
        Self: Sized;

    async fn add(&mut self, document: Document) -> Result<bool>;
    async fn retrieve(&self, query: &str, top_k: usize) -> Result<Vec<(Document, f64)>>;
}

pub async fn factory(
    flavor: &str,
    embedder: Box<dyn Embedder>,
    config: Configuration,
) -> Result<Box<dyn VectorStore>> {
    match flavor {
        "naive" => Ok(Box::new(NaiveVectorStore::new(embedder, config).await?)),
        _ => Err(anyhow!("flavor '{flavor} not supported yet")),
    }
}
