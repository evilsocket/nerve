use std::{collections::HashMap, path::PathBuf, time::Instant};

use rayon::prelude::*;

use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;
use glob::glob;
use serde::{Deserialize, Serialize};

use crate::{metrics, Embedder};

use super::{Configuration, Document, Embeddings, VectorStore};

#[derive(Serialize, Deserialize)]
struct Store {
    documents: HashMap<String, Document>,
    embeddings: HashMap<String, Embeddings>,
}

impl Store {
    fn new() -> Self {
        let documents = HashMap::new();
        let embeddings = HashMap::new();
        Self {
            documents,
            embeddings,
        }
    }

    fn from_data_path(path: &str) -> Result<Self> {
        let path = PathBuf::from(path).join("rag.bin");
        if path.exists() {
            let raw = std::fs::read(&path)?;
            Ok(bitcode::deserialize(&raw)?)
        } else {
            Ok(Store::new())
        }
    }

    fn to_data_path(&self, path: &str) -> Result<()> {
        let path = PathBuf::from(path).join("rag.bin");
        let raw = bitcode::serialize(&self)?;

        std::fs::write(path, raw)?;

        Ok(())
    }
}

pub struct NaiveVectorStore {
    config: Configuration,
    embedder: Box<dyn Embedder>,
    store: Store,
}

impl NaiveVectorStore {
    fn from_data_path(embedder: Box<dyn Embedder>, config: Configuration) -> Result<Self> {
        let store = Store::from_data_path(&config.data_path)?;
        Ok(Self {
            config,
            embedder,
            store,
        })
    }

    async fn import_new_documents(&mut self) -> Result<()> {
        let path = std::fs::canonicalize(&self.config.source_path)?
            .display()
            .to_string();
        let expr = format!("{}/**/*.txt", path);
        let start = Instant::now();
        let mut new = 0;

        for path in (glob(&expr)?).flatten() {
            let docs = if let Some(chunk_size) = self.config.chunk_size {
                Document::from_text_file(&path)?.chunks(chunk_size)?
            } else {
                vec![Document::from_text_file(&path)?]
            };

            for doc in docs {
                match self.add(doc).await {
                    Err(err) => eprintln!("ERROR storing {}: {}", path.display(), err),
                    Ok(added) => {
                        if added {
                            new += 1
                        }
                    }
                }
            }
        }

        if new > 0 {
            println!(
                "[{}] {} new documents indexed in {:?}\n",
                "rag".bold(),
                new,
                start.elapsed(),
            );
        }

        Ok(())
    }
}

#[async_trait]
impl VectorStore for NaiveVectorStore {
    #[allow(clippy::borrowed_box)]
    async fn new(embedder: Box<dyn Embedder>, config: Configuration) -> Result<Self>
    where
        Self: Sized,
    {
        let mut store = Self::from_data_path(embedder, config)?;

        store.import_new_documents().await?;

        Ok(store)
    }

    async fn add(&mut self, mut document: Document) -> Result<bool> {
        let doc_id = document.get_ident().to_string();
        let doc_path = document.get_path().to_string();

        if self.store.documents.contains_key(&doc_id) {
            // println!("document with id '{}' already indexed", &doc_id);
            return Ok(false);
        }

        print!(
            "[{}] indexing new document '{}' ({} bytes) ...",
            "rag".bold(),
            doc_path,
            document.get_byte_size()?
        );

        let start = Instant::now();
        let embeddings: Vec<f64> = self.embedder.embed(document.get_data()?).await?;
        let size = embeddings.len();

        // get rid of the contents once indexed
        document.drop_data();

        self.store.documents.insert(doc_id.to_string(), document);
        self.store.embeddings.insert(doc_id, embeddings);

        self.store.to_data_path(&self.config.data_path)?;

        println!(" time={:?} embedding_size={}", start.elapsed(), size);

        Ok(true)
    }

    async fn retrieve(&self, query: &str, top_k: usize) -> Result<Vec<(Document, f64)>> {
        println!("[{}] {} (top {})", "rag".bold(), query, top_k);

        let query_vector = self.embedder.embed(query).await?;
        let mut results = vec![];

        let distances: Vec<(&String, f64)> = {
            let mut distances: Vec<(&String, f64)> = self
                .store
                .embeddings
                .par_iter()
                .map(|(doc_id, doc_embedding)| {
                    (doc_id, metrics::cosine(&query_vector, doc_embedding))
                })
                .collect();
            distances.par_sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());
            distances
        };

        for (doc_id, score) in distances {
            let document = self.store.documents.get(doc_id).unwrap();
            results.push((document.clone(), score));
            if results.len() >= top_k {
                break;
            }
        }

        Ok(results)
    }
}
