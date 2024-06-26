use std::{collections::HashMap, time::Instant};

#[cfg(feature = "rayon")]
use rayon::prelude::*;

use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;
use glob::glob;

use super::{Configuration, Document, Embeddings, VectorStore};
use crate::agent::{generator::Client, rag::metrics};

// TODO: integrate other more efficient vector databases.

pub struct NaiveVectorStore {
    config: Configuration,
    embedder: Box<dyn Client>,
    documents: HashMap<String, Document>,
    embeddings: HashMap<String, Embeddings>,
}

#[async_trait]
impl VectorStore for NaiveVectorStore {
    #[allow(clippy::borrowed_box)]
    async fn new(embedder: Box<dyn Client>, config: Configuration) -> Result<Self>
    where
        Self: Sized,
    {
        // TODO: add persistency
        let documents = HashMap::new();
        let embeddings = HashMap::new();
        let mut store = Self {
            config,
            documents,
            embeddings,
            embedder,
        };

        let path = std::fs::canonicalize(&store.config.path)?
            .display()
            .to_string();
        let expr = format!("{}/**/*.txt", path);

        for path in (glob(&expr)?).flatten() {
            let doc = Document::from_text_file(&path)?;
            if let Err(err) = store.add(doc).await {
                eprintln!("ERROR storing {}: {}", path.display(), err);
            }
        }

        Ok(store)
    }

    async fn add(&mut self, document: Document) -> Result<()> {
        let doc_path = document.get_path().to_string();

        if self.documents.contains_key(&doc_path) {
            return Err(anyhow!(
                "document with name '{}' already indexed",
                &doc_path
            ));
        }

        // TODO: add chunking
        print!(
            "[{}] indexing document '{}' ({} bytes) ...",
            "rag".bold(),
            &doc_path,
            document.get_byte_size()
        );

        let start = Instant::now();
        let embeddings: Vec<f64> = self.embedder.embeddings(document.get_data()).await?;
        let size = embeddings.len();

        self.documents.insert(doc_path.to_string(), document);
        self.embeddings.insert(doc_path, embeddings);

        println!(" time={:?} embedding_size={}", start.elapsed(), size);

        Ok(())
    }

    async fn retrieve(&self, query: &str, top_k: usize) -> Result<Vec<(Document, f64)>> {
        println!("[{}] {} (top {})", "rag".bold(), query, top_k);

        let query_vector = self.embedder.embeddings(query).await?;
        let mut results = vec![];

        #[cfg(feature = "rayon")]
        let distances: Vec<(&String, f64)> = {
            let mut distances: Vec<(&String, f64)> = self
                .embeddings
                .par_iter()
                .map(|(doc_name, doc_embedding)| {
                    (doc_name, metrics::cosine(&query_vector, doc_embedding))
                })
                .collect();
            distances.par_sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());
            distances
        };

        #[cfg(not(feature = "rayon"))]
        let distances = {
            let mut distances = vec![];
            for (doc_name, doc_embedding) in &self.embeddings {
                distances.push((doc_name, metrics::cosine(&query_vector, doc_embedding)));
            }
            distances.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());
            distances
        };

        for (doc_name, score) in distances {
            let document = self.documents.get(doc_name).unwrap();
            results.push((document.clone(), score));
            if results.len() >= top_k {
                break;
            }
        }

        Ok(results)
    }
}
