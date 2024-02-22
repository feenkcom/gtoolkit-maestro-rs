use downloader::{FileToDownload, FilesToDownload};
use unzipper::{FileToUnzip, FilesToUnzip};

use crate::{Application, PlatformOS, Result, DOWNLOADING, EXTRACTING};

pub struct Downloader {
    silent: bool,
}

impl Downloader {
    pub fn new() -> Self {
        Self { silent: false }
    }

    pub fn be_silent(mut self) -> Self {
        self.silent = true;
        self
    }

    pub fn gtoolkit_vm_to_download(
        application: &Application,
        target: PlatformOS,
    ) -> FileToDownload {
        let suffix = if application.host_platform() != target {
            format!("-{}", target.as_str())
        } else {
            "".to_string()
        };

        let extension = if target == PlatformOS::AndroidAarch64 {
            "apk"
        } else {
            "zip"
        };

        let file_name = format!(
            "GlamorousToolkitApp{}-v{}.{}",
            suffix,
            application.app_version().to_string(),
            extension
        );

        FileToDownload::new(
            application.gtoolkit_app_url_for_target(target),
            application.gtoolkit_app_location(target),
            file_name,
        )
    }

    pub fn files_to_download(application: &Application, target: PlatformOS) -> FilesToDownload {
        let files_to_download = FilesToDownload::new();
        if application.has_explicit_app_cli_binary() {
            files_to_download
        } else {
            files_to_download.add(Self::gtoolkit_vm_to_download(application, target))
        }
    }

    pub fn files_to_unzip(application: &Application, target: PlatformOS) -> FilesToUnzip {
        let files_to_unzip = FilesToUnzip::new();
        if application.has_explicit_app_cli_binary() {
            files_to_unzip
        } else {
            let gtoolkit_vm = Self::gtoolkit_vm_to_download(application, target);
            files_to_unzip.add(FileToUnzip::new(
                gtoolkit_vm.path(),
                application.gtoolkit_app_location(target),
            ))
        }
    }

    pub async fn download_glamorous_toolkit_vm(
        &self,
        application: &Application,
        target: PlatformOS,
    ) -> Result<()> {
        if !self.silent {
            println!(
                "{}Downloading GlamorousToolkit App (v{}, {})...",
                DOWNLOADING,
                application.app_version().to_string(),
                target.as_str()
            );
        }

        Self::files_to_download(application, target)
            .download()
            .await?;

        if !self.silent {
            println!(
                "{}Extracting GlamorousToolkit App (v{})...",
                EXTRACTING,
                application.app_version().to_string()
            );
        }

        Self::files_to_unzip(application, target).unzip().await?;

        Ok(())
    }
}
