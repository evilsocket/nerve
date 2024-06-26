use std::path::Path;

use anyhow::Result;

// #[cfg(feature = "rayon")]
// use rayon::prelude::*;

// const CHUNK_SIZE: usize = 1024;

#[derive(Clone, Debug)]
pub(crate) struct Document {
    path: String,
    data: String,
}

impl Document {
    pub fn from_text_file(path: &Path) -> Result<Self> {
        let path = std::fs::canonicalize(path.display().to_string())?
            .display()
            .to_string();
        let data = std::fs::read_to_string(&path)?;
        Ok(Self { path, data })
    }

    pub fn get_path(&self) -> &str {
        &self.path
    }

    pub fn get_data(&self) -> &str {
        &self.data
    }

    pub fn get_byte_size(&self) -> usize {
        self.data.as_bytes().len()
    }

    /*
    pub fn as_chunks(self) -> Vec<Document> {
        #[cfg(feature = "rayon")]
        return self
            .data
            .chars()
            .collect::<Vec<char>>()
            .par_chunks(CHUNK_SIZE)
            .enumerate()
            .map(|(idx, c)| Document {
                path: format!("{}@{}", self.path, idx),
                data: c.iter().collect::<String>(),
            })
            .collect();

        #[cfg(not(feature = "rayon"))]
        return self
            .data
            .chars()
            .collect::<Vec<char>>()
            .chunks(CHUNK_SIZE)
            .enumerate()
            .map(|(idx, c)| Document {
                path: format!("{}@{}", self.path, idx),
                data: c.iter().collect::<String>(),
            })
            .collect();
    }
     */
}
