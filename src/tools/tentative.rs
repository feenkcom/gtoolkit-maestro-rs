use clap::Parser;
use std::path::PathBuf;

use file_matcher::FileNamed;
use unzipper::{FileToUnzip, FilesToUnzip};
use zipper::ToZip;

use crate::{Application, Downloader, Result};

#[derive(Parser, Debug, Clone)]
pub struct TentativeOptions {
    /// Path to the .zip with the tentative image build
    #[clap(parse(from_os_str))]
    pub tentative: PathBuf,
    /// When packaging or un-packaging, do not fail when some of the items do not exist.
    /// This may be useful when packaging a local build.
    #[clap(long)]
    pub ignore_absent: bool,
}

pub struct Tentative;

impl Tentative {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn package(
        &self,
        application: &Application,
        tentative_options: &TentativeOptions,
    ) -> Result<PathBuf> {
        let mut zip = ToZip::new(tentative_options.tentative.as_path())
            .one_entry(FileNamed::wildmatch("*.image").within(application.workspace()))
            .one_entry(FileNamed::wildmatch("*.changes").within(application.workspace()))
            .one_entry(FileNamed::wildmatch("*.sources").within(application.workspace()))
            .one_entry(
                FileNamed::exact(application.serialization_file_name())
                    .within(application.workspace()),
            )
            .folder(application.workspace().join("gt-extra"))
            .one_entries(application.gtoolkit_app_folders());

        let gt_extra = application.workspace().join("gt-extra");

        if gt_extra.exists() || !tentative_options.ignore_absent {
            zip.add_folder(gt_extra);
        }

        zip.zip().map_err(|error| error.into())
    }

    pub async fn unpackage(
        &self,
        application: &mut Application,
        tentative_options: &TentativeOptions,
    ) -> Result<()> {
        let files_to_unzip = FilesToUnzip::new().add(FileToUnzip::new(
            tentative_options.tentative.as_path(),
            application.workspace(),
        ));

        files_to_unzip.unzip().await?;

        application.deserialize_from_file()?;

        Downloader::new()
            .download_glamorous_toolkit_vm(&application)
            .await?;

        Ok(())
    }
}
