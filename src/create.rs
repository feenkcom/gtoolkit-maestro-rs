use crate::Result;
use std::path::PathBuf;

pub struct FileToCreate {
    content: String,
    destination: PathBuf,
}

impl FileToCreate {
    pub fn new(path: impl Into<PathBuf>, content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            destination: path.into(),
        }
    }

    pub async fn create(&self) -> Result<()> {
        tokio::fs::write(&self.destination, &self.content).await?;
        Ok(())
    }
}
