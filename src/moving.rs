use crate::error::Error;
use regex::Regex;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FileToMove {
    pattern: String,
    directory: PathBuf,
    destination: PathBuf,
}

impl FileToMove {
    pub fn new(
        pattern: impl Into<String>,
        from: impl Into<PathBuf>,
        destination: impl Into<PathBuf>,
    ) -> Self {
        Self {
            pattern: pattern.into(),
            directory: from.into(),
            destination: destination.into(),
        }
    }

    async fn find_file_matching(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let mut entries = tokio::fs::read_dir(&self.directory).await?;
        let regex = Regex::new(&self.pattern).unwrap();

        let mut matches: Vec<PathBuf> = vec![];
        while let Some(entry) = entries.next_entry().await? {
            if entry.path().is_file() {
                if let Some(filename) = entry.path().file_name() {
                    if let Some(filename) = filename.to_str() {
                        if regex.is_match(filename) {
                            matches.push(entry.path().to_path_buf())
                        }
                    }
                }
            }
        }

        match matches.len() {
            0 => Err(Error {
                what: format!(
                    "Could not find a file matching {} in {:?}",
                    &self.pattern, &self.directory
                ),
                source: None,
            }
            .into()),
            1 => Ok(matches.get(0).unwrap().clone()),
            _ => Err(Error {
                what: format!(
                    "Found more than one file matching {} in {:?}",
                    &self.pattern, &self.directory
                ),
                source: None,
            }
            .into()),
        }
    }

    pub async fn move_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        let file = self.find_file_matching().await?;

        let file_name = file.file_name().unwrap().to_str().unwrap();

        if self.destination.is_dir() {
            tokio::fs::copy(&file, &self.destination.join(file_name)).await?;
        } else {
            tokio::fs::copy(&file, &self.destination).await?;
        }
        tokio::fs::remove_file(&file).await?;
        Ok(())
    }
}
