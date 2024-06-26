use std::path::PathBuf;

use anyhow::Result;

#[derive(Clone, Debug)]
pub(crate) struct Document {
    path: String,
    data: String,
}

impl Document {
    pub fn from_text_file(path: &PathBuf) -> Result<Self> {
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
}
