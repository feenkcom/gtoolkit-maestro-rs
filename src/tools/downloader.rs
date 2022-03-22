use unzipper::{FilesToUnzip, FileToUnzip};
use crate::download::{FileToDownload, FilesToDownload};
use crate::{Application, Result, DOWNLOADING, EXTRACTING};

pub struct Downloader;

impl Downloader {
    pub fn new() -> Self {
        Self {}
    }

    pub fn gtoolkit_vm_to_download(application: &Application) -> FileToDownload {
        FileToDownload::new(
            application.gtoolkit_app_url(),
            application.workspace(),
            format!(
                "GlamorousToolkitApp-v{}.zip",
                application.app_version().to_string()
            ),
        )
    }

    pub fn files_to_download(application: &Application) -> FilesToDownload {
        FilesToDownload::new().add(Self::gtoolkit_vm_to_download(application))
    }

    pub fn files_to_unzip(application: &Application) -> FilesToUnzip {
        let gtoolkit_vm = Self::gtoolkit_vm_to_download(application);
        FilesToUnzip::new().add(FileToUnzip::new(
            gtoolkit_vm.path(),
            application.workspace(),
        ))
    }

    pub async fn download_glamorous_toolkit_vm(&self, application: &Application) -> Result<()> {
        println!(
            "{}Downloading GlamorousToolkit App (v{})...",
            DOWNLOADING,
            application.app_version().to_string()
        );

        Self::files_to_download(application).download().await?;

        println!(
            "{}Extracting GlamorousToolkit App (v{})...",
            EXTRACTING,
            application.app_version().to_string()
        );

        Self::files_to_unzip(application).unzip().await?;

        Ok(())
    }
}
