use crate::options::AppOptions;
use clap::{AppSettings, Clap};
use std::path::PathBuf;

use file_matcher::{FileNamed, OneFileFilter};

use crate::{zip_file, zip_folder, Checker, Downloader, FileToUnzip, FilesToUnzip};

#[derive(Clap, Debug, Clone)]
#[clap(setting = AppSettings::ColorAlways)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct TentativeOptions {
    /// Path to the .zip with the tentative image build
    #[clap(parse(from_os_str))]
    pub tentative: PathBuf,
}

pub struct Tentative;

impl Tentative {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn package(
        &self,
        options: &AppOptions,
        tentative_options: &TentativeOptions,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file = std::fs::File::create(&tentative_options.tentative).unwrap();
        let mut zip = zip::ZipWriter::new(file);

        let zip_options =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        let filters = vec![
            FileNamed::wildmatch("*.image"),
            FileNamed::wildmatch("*.changes"),
            FileNamed::wildmatch("*.sources"),
            FileNamed::exact(options.vm_version_file_name()),
        ]
        .into_iter()
        .map(|each| each.within(options.gtoolkit_directory()))
        .collect::<Vec<OneFileFilter>>();

        for ref filter in filters {
            zip_file(&mut zip, filter.as_path_buf()?, zip_options)?;
        }

        zip_folder(
            &mut zip,
            options.gtoolkit_directory().join("gt-extra"),
            zip_options,
        )?;

        zip.finish()?;

        Ok(())
    }

    pub async fn unpackage(
        &self,
        options: &mut AppOptions,
        tentative_options: &TentativeOptions,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Checker::new().check(&options, false).await?;

        let files_to_unzip = FilesToUnzip::new().add(FileToUnzip::new(
            tentative_options.tentative.as_path(),
            options.gtoolkit_directory(),
        ));

        files_to_unzip.unzip().await?;

        options.read_vm_version().await?;

        Downloader::new()
            .download_glamorous_toolkit_vm(&options)
            .await?;

        Ok(())
    }
}
