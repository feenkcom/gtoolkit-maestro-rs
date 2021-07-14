use regex::Regex;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FolderToMove {
    folder: PathBuf,
    destination: PathBuf,
}

impl FolderToMove {
    pub fn new(folder: impl Into<PathBuf>, destination: impl Into<PathBuf>) -> Self {
        Self {
            folder: folder.into(),
            destination: destination.into(),
        }
    }

    pub async fn move_folder(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut stack = Vec::new();
        stack.push(self.folder.clone());

        let output_root = self.destination.clone();
        let input_root = self.folder.components().count();

        while let Some(working_path) = stack.pop() {
            // Generate a relative path
            let src: PathBuf = working_path.components().skip(input_root).collect();

            // Create a destination if missing
            let dest = if src.components().count() == 0 {
                output_root.clone()
            } else {
                output_root.join(&src)
            };
            if tokio::fs::metadata(&dest).await.is_err() {
                tokio::fs::create_dir_all(&dest).await?;
            }

            let mut entries = tokio::fs::read_dir(working_path).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else {
                    match path.file_name() {
                        Some(filename) => {
                            let dest_path = dest.join(filename);
                            tokio::fs::copy(&path, &dest_path).await?;
                        }
                        None => {}
                    }
                }
            }
        }

        Ok(())
    }
}
