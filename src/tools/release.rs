use crate::options::AppOptions;
use crate::{zip_file, zip_folder};
use clap::{AppSettings, Clap};
use file_matcher::{FileNamed, OneEntry};
use std::path::PathBuf;

#[derive(Clap, Debug, Clone)]
#[clap(setting = AppSettings::ColorAlways)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct ReleaseOptions {
    /// Path to the .zip with the release image build
    #[clap(parse(from_os_str))]
    pub release: PathBuf,
}

pub struct Release;

impl Release {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn package(
        &self,
        options: &AppOptions,
        release_options: &ReleaseOptions,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file = std::fs::File::create(&release_options.release).unwrap();
        let mut zip = zip::ZipWriter::new(file);

        let zip_options =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        let filters = vec![
            FileNamed::wildmatch("*.image"),
            FileNamed::wildmatch("*.changes"),
            FileNamed::wildmatch("*.sources"),
        ]
        .into_iter()
        .map(|each| each.within(options.gtoolkit_directory()))
        .collect::<Vec<OneEntry>>();

        for ref filter in filters {
            zip_file(&mut zip, filter.as_path_buf()?, zip_options)?;
        }

        let gt_extra = options.gtoolkit_directory().join("gt-extra");
        zip_folder(&mut zip, &gt_extra, zip_options)?;

        for folder in options.gtoolkit_app_folders() {
            zip_folder(&mut zip, folder.as_path_buf()?, zip_options)?;
        }

        zip.finish()?;

        Ok(())
    }
}
