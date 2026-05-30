use clap::ArgEnum;
use downloader::{FileToDownload, FilesToDownload};
use feenk_download_auth_client::{
    download_release_asset_with_env_auth, EnvDownloadRequest, InstallationTokenSource,
};
use std::env;
use unzipper::{FileToUnzip, FilesToUnzip};

use crate::options::VM_PRO_REPOSITORY_NAME;
use crate::{Application, InstallerError, PlatformOS, Result, DOWNLOADING, EXTRACTING};

const FEENK_DOWNLOAD_AUTH_SERVER_URL: &str = "https://dl-auth.feenk.com";
const FEENK_CUSTOMER_ID_ENV: &str = "FEENK_CUSTOMER_ID";
const FEENK_CUSTOMER_KEY_ENV: &str = "FEENK_CUSTOMER_KEY";

pub struct Downloader {
    silent: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ArgEnum)]
pub enum CustomerLevel {
    Auto,
    Regular,
    Pro,
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

    pub fn should_download_pro_vm(customer_level: CustomerLevel) -> Result<bool> {
        let has_credentials = match (
            env::var_os(FEENK_CUSTOMER_ID_ENV),
            env::var_os(FEENK_CUSTOMER_KEY_ENV),
        ) {
            (Some(_), Some(_)) => Ok(true),
            (None, None) => Ok(false),
            _ => InstallerError::ProVmCredentialsConfigurationError.into(),
        }?;

        match customer_level {
            CustomerLevel::Auto => Ok(has_credentials),
            CustomerLevel::Regular => Ok(false),
            CustomerLevel::Pro => {
                if has_credentials {
                    Ok(true)
                } else {
                    InstallerError::ProVmCredentialsRequired.into()
                }
            }
        }
    }

    pub fn files_to_download(
        application: &Application,
        target: PlatformOS,
        customer_level: CustomerLevel,
    ) -> Result<FilesToDownload> {
        let files_to_download = FilesToDownload::new();
        let files_to_download = if application.has_explicit_app_cli_binary()
            || Self::should_download_pro_vm(customer_level)?
        {
            files_to_download
        } else {
            files_to_download.add(Self::gtoolkit_vm_to_download(application, target))
        };
        Ok(files_to_download)
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

        self.download_glamorous_toolkit_vm_archive(application, target, CustomerLevel::Auto)
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

    pub async fn download_glamorous_toolkit_vm_archive(
        &self,
        application: &Application,
        target: PlatformOS,
        customer_level: CustomerLevel,
    ) -> Result<()> {
        if application.has_explicit_app_cli_binary() {
            return Ok(());
        }

        let gtoolkit_vm = Self::gtoolkit_vm_to_download(application, target);

        if Self::should_download_pro_vm(customer_level)? {
            let asset_name = application.gtoolkit_pro_app_file_name_for_target(target);
            let tag = format!("v{}", application.app_version());
            let output_path = gtoolkit_vm.path();

            if let Some(output_directory) = output_path.parent() {
                std::fs::create_dir_all(output_directory)?;
            }

            download_release_asset_with_env_auth(EnvDownloadRequest {
                token_source: InstallationTokenSource::customer_env(
                    FEENK_DOWNLOAD_AUTH_SERVER_URL,
                    FEENK_CUSTOMER_ID_ENV,
                    FEENK_CUSTOMER_KEY_ENV,
                ),
                repo: VM_PRO_REPOSITORY_NAME.to_string(),
                github_owner: None,
                tag: Some(tag),
                asset_name,
                output_path,
            })
            .await?;

            Ok(())
        } else {
            FilesToDownload::new().add(gtoolkit_vm).download().await?;
            Ok(())
        }
    }
}
