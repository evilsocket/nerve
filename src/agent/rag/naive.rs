use std::{collections::HashMap, time::Instant};

use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;
use glob::glob;

use super::{Document, Embeddings, VectorStore};
use crate::agent::{generator::Client, rag::metrics};

pub struct NaiveVectorStore {
    generator: Box<dyn Client>,
    documents: HashMap<String, Document>,
    embeddings: HashMap<String, Embeddings>,
}

impl NaiveVectorStore {
    pub async fn from_indexed_path(generator: Box<dyn Client>, path: &str) -> Result<Self> {
        let path = std::fs::canonicalize(path)?.display().to_string();
        let expr = format!("{}/**/*.txt", path);
        let mut store = NaiveVectorStore::new_with_generator(generator)?;

        for path in (glob(&expr)?).flatten() {
            let doc_name = path.display();
            let doc = Document {
                name: doc_name.to_string(),
                data: std::fs::read_to_string(&path)?,
            };
            if let Err(err) = store.add(doc).await {
                eprintln!("ERROR storing {}: {}", doc_name, err);
            }
        }

        Ok(store)
    }
}

#[async_trait]
impl VectorStore for NaiveVectorStore {
    #[allow(clippy::borrowed_box)]
    fn new_with_generator(generator: Box<dyn Client>) -> Result<Self>
    where
        Self: Sized,
    {
        let documents = HashMap::new();
        let embeddings = HashMap::new();

        Ok(Self {
            documents,
            embeddings,
            generator,
        })
    }

    async fn add(&mut self, document: Document) -> Result<()> {
        if self.documents.contains_key(&document.name) {
            return Err(anyhow!(
                "document with name '{}' already indexed",
                &document.name
            ));
        }

        // TODO: add chunking
        let data_size = document.data.as_bytes().len();

        println!(
            "[{}] indexing document '{}' ({} bytes) ...",
            "rag".bold(),
            &document.name,
            data_size
        );

        let start = Instant::now();
        let doc_name = document.name.to_string();
        let embeddings = self.generator.embeddings(&document.data).await?;

        self.documents.insert(doc_name.to_string(), document);
        self.embeddings.insert(doc_name, embeddings);

        println!("[rag] document indexed in {:?}", start.elapsed(),);

        Ok(())
    }

    async fn retrieve(&self, query: &str, top_k: usize) -> Result<Vec<(Document, f64)>> {
        println!("[{}] {} (top {})", "rag".bold(), query, top_k);

        let query_vector = self.generator.embeddings(query).await?;
        let mut distances = vec![];
        let mut results = vec![];

        // TODO: parallelize?
        for (doc_name, doc_embedding) in &self.embeddings {
            distances.push((doc_name, metrics::cosine(&query_vector, doc_embedding)));
        }

        distances.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());

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
