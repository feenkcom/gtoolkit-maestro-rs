use clap::Parser;
use std::path::PathBuf;

use file_matcher::{FileNamed, OneEntry};

use crate::{zip_file, zip_folder, Application, Downloader, FileToUnzip, FilesToUnzip, Result};

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
    ) -> Result<()> {
        let file = std::fs::File::create(&tentative_options.tentative).unwrap();
        let mut zip = zip::ZipWriter::new(file);

        let zip_options =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        let filters = vec![
            FileNamed::wildmatch("*.image"),
            FileNamed::wildmatch("*.changes"),
            FileNamed::wildmatch("*.sources"),
            FileNamed::exact(application.serialization_file_name()),
        ]
        .into_iter()
        .map(|each| each.within(application.workspace()))
        .collect::<Vec<OneEntry>>();

        for ref filter in filters {
            zip_file(&mut zip, filter.as_path_buf()?, zip_options)?;
        }

        let gt_extra = application.workspace().join("gt-extra");

        if gt_extra.exists() || !tentative_options.ignore_absent {
            zip_folder(&mut zip, &gt_extra, zip_options)?;
        }

        zip.finish()?;

        Ok(())
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
