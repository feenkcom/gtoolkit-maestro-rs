use crate::download::{FileToDownload, FilesToDownload};
use crate::options::AppOptions;
use crate::{FileToUnzip, FilesToUnzip, DOWNLOADING, EXTRACTING};

pub struct Downloader;

impl Downloader {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn download_glamorous_toolkit_vm(
        &self,
        options: &AppOptions,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "{}Downloading GlamorousToolkit App (v{})...",
            DOWNLOADING,
            options.gtoolkit_app_version_string()
        );
        let gtoolkit_vm = FileToDownload::new(
            options.gtoolkit_app_url(),
            options.workspace(),
            format!(
                "GlamorousToolkitApp-v{}.zip",
                options.gtoolkit_app_version_string()
            ),
        );

        let files_to_download = FilesToDownload::new().add(gtoolkit_vm.clone());

        files_to_download.download().await?;

        println!(
            "{}Extracting GlamorousToolkit App (v{})...",
            EXTRACTING,
            options.gtoolkit_app_version_string()
        );
        let files_to_unzip =
            FilesToUnzip::new().add(FileToUnzip::new(gtoolkit_vm.path(), options.workspace()));

        files_to_unzip.unzip().await?;

        Ok(())
    }
}
