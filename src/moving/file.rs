use crate::Result;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FileToMove {
    file: PathBuf,
    destination: PathBuf,
}

impl FileToMove {
    pub fn new(file: impl Into<PathBuf>, destination: impl Into<PathBuf>) -> Self {
        Self {
            file: file.into(),
            destination: destination.into(),
        }
    }

    pub async fn move_file(&self) -> Result<()> {
        let file_name = self.file.file_name().unwrap().to_str().unwrap();

        if self.destination.is_dir() {
            tokio::fs::copy(&self.file, &self.destination.join(file_name)).await?;
        } else {
            tokio::fs::copy(&self.file, &self.destination).await?;
        }
        tokio::fs::remove_file(&self.file).await?;
        Ok(())
    }
}
