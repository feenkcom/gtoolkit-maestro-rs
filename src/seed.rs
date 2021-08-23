use crate::FileToUnzip;
use crate::{Application, FileToDownload};
use std::path::PathBuf;
use url::Url;

/// Represents a seed from which to build am image.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ImageSeed {
    Url(Url),
    Zip(PathBuf),
    Image(PathBuf),
}

impl ImageSeed {
    pub fn file_to_download(&self, application: &Application) -> Option<FileToDownload> {
        match self {
            Self::Url(url) => Some(FileToDownload::new(
                url.to_string(),
                application.workspace(),
                "seed-image.zip",
            )),
            _ => None,
        }
    }

    pub fn file_to_unzip(&self, application: &Application) -> Option<FileToUnzip> {
        match self {
            Self::Url(_) => Some(FileToUnzip::new(
                application.workspace().join("seed-image.zip"),
                self.seed_image_directory(application),
            )),
            Self::Zip(zip_archive) => Some(FileToUnzip::new(
                zip_archive,
                self.seed_image_directory(application),
            )),
            _ => None,
        }
    }

    pub fn seed_image_directory(&self, application: &Application) -> PathBuf {
        match self {
            Self::Image(image_file) => image_file
                .parent()
                .expect("Failed to get a parent directory of the image")
                .to_path_buf(),
            _ => application.workspace().join("seed-image"),
        }
    }

    pub fn target_image_directory(&self, application: &Application) -> PathBuf {
        match self {
            Self::Image(_) => self.seed_image_directory(application),
            _ => application.workspace().to_path_buf(),
        }
    }

    pub fn is_image_file(&self) -> bool {
        match self {
            Self::Image(_) => true,
            _ => false,
        }
    }
}
