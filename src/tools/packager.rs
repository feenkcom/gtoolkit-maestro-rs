use crate::options::AppOptions;

use file_matcher::{FileNamed, OneFileFilter};

use crate::{zip_file, zip_folder};

pub struct Packager;

impl Packager {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn package_image(&self, options: &AppOptions) -> Result<(), Box<dyn std::error::Error>> {
        let file = std::fs::File::create(
            options
                .gtoolkit_directory()
                .join("GlamorousToolkit-tentative.zip"),
        )
        .unwrap();
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
}
