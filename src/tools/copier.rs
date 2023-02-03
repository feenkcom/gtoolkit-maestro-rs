use clap::Parser;
use std::path::PathBuf;

use crate::{Application, Result};
use file_matcher::{FileNamed, FolderNamed, OneEntry, OneEntryCopier, OneEntryNamed};

#[derive(Parser, Debug, Clone)]
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
        application: &mut Application,
        copy_options: &CopyOptions,
    ) -> Result<()> {
        let mut entries = vec![
            FileNamed::wildmatch("*.image").boxed(),
            FileNamed::wildmatch("*.changes").boxed(),
            FileNamed::wildmatch("*.sources").boxed(),
            FileNamed::exact(Application::serialization_file_name()).boxed(),
            FolderNamed::exact("gt-extra").boxed(),
        ];

        entries.extend(application.gtoolkit_app_entries());

        let entries = entries
            .into_iter()
            .map(|each| each.within_path_buf(application.workspace().to_path_buf()))
            .collect::<Vec<OneEntry>>();

        if !copy_options.destination.exists() {
            std::fs::create_dir_all(copy_options.destination.as_path())?;
        }

        for ref entry in entries {
            entry.copy(copy_options.destination.as_path())?;
        }

        application.set_workspace(copy_options.destination.clone());

        Ok(())
    }
}
