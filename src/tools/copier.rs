use crate::options::AppOptions;
use clap::{AppSettings, Clap};
use std::path::PathBuf;

use file_matcher::{FileNamed, FolderNamed, OneEntry, OneEntryCopier, OneEntryNamed};

#[derive(Clap, Debug, Clone)]
#[clap(setting = AppSettings::ColorAlways)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct CopyOptions {
    /// A folder in which to copy the image, changes and sources with some extra files
    #[clap(parse(from_os_str), default_value = crate::options::DEFAULT_DIRECTORY)]
    pub destination: PathBuf,
}

pub struct Copier;

impl Copier {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn copy(
        &self,
        options: &mut AppOptions,
        copy_options: &CopyOptions,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut entries = vec![
            FileNamed::wildmatch("*.image").boxed(),
            FileNamed::wildmatch("*.changes").boxed(),
            FileNamed::wildmatch("*.sources").boxed(),
            FileNamed::exact(options.vm_version_file_name()).boxed(),
            FileNamed::exact(options.gtoolkit_version_file_name()).boxed(),
            FolderNamed::exact("gt-extra").boxed(),
        ];

        entries.extend(options.gtoolkit_app_entries());

        let entries = entries
            .into_iter()
            .map(|each| each.within_path_buf(options.workspace()))
            .collect::<Vec<OneEntry>>();

        if !copy_options.destination.exists() {
            std::fs::create_dir_all(copy_options.destination.as_path())?;
        }

        for ref entry in entries {
            entry.copy(copy_options.destination.as_path())?;
        }

        options.set_workspace(copy_options.destination.clone());

        Ok(())
    }
}
