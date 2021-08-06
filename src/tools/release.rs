use crate::options::{AppOptions, PlatformOS};
use crate::{zip_file, zip_folder};
use clap::{AppSettings, Clap};
use feenk_releaser::VersionBump;
use file_matcher::{FileNamed, OneEntry};
use std::path::{Path, PathBuf};

#[derive(Clap, Debug, Clone)]
#[clap(setting = AppSettings::ColorAlways)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct ReleaseOptions {
    /// Path to the .zip with the release image build. Supports mustache syntax to inject various release related information.
    /// The following properties are supported:
    /// - {{version}} - the release version in a form of X.Y.Z
    /// - {{os}} - the OS we release for. (`MacOS`, `Linux`, `Windows`)
    /// - {{arch}} - the target release architecture. (`x86_64`, `aarch64`)
    #[clap(parse(from_os_str))]
    pub release: PathBuf,
    #[clap(long, default_value = VersionBump::Patch.to_str(), possible_values = VersionBump::variants(), case_insensitive = true)]
    pub bump: VersionBump,
}

#[derive(Serialize)]
struct ReleaseInfo {
    version: String,
    os: String,
    arch: String,
}

pub struct Release;

impl Release {
    pub fn new() -> Self {
        Self {}
    }

    fn process_template_path(
        options: &AppOptions,
        release_options: &ReleaseOptions,
        path: impl AsRef<Path>,
    ) -> PathBuf {
        let new_version = options
            .gtoolkit_version()
            .expect("Must have a gtoolkit version")
            .bump(release_options.bump.clone());
        let platform = match options.platform() {
            PlatformOS::MacOSX8664 => "MacOS",
            PlatformOS::MacOSAarch64 => "MacOS",
            PlatformOS::WindowsX8664 => "Windows",
            PlatformOS::LinuxX8664 => "Linux",
        };

        let arch = match options.platform() {
            PlatformOS::MacOSX8664 => "x86_64",
            PlatformOS::MacOSAarch64 => "aarch64",
            PlatformOS::WindowsX8664 => "x86_64",
            PlatformOS::LinuxX8664 => "x86_64",
        };

        let info = ReleaseInfo {
            version: new_version.to_string(),
            os: platform.to_string(),
            arch: arch.to_string(),
        };

        path.as_ref()
            .iter()
            .map(|each| {
                let template = mustache::compile_str(each.to_str().unwrap()).unwrap();
                template.render_to_string(&info).unwrap()
            })
            .collect::<PathBuf>()
    }

    pub async fn package(
        &self,
        options: &AppOptions,
        release_options: &ReleaseOptions,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let package = Self::process_template_path(
            options,
            release_options,
            release_options.release.as_path(),
        );

        let file = std::fs::File::create(&package).unwrap();
        let mut zip = zip::ZipWriter::new(file);

        let zip_options =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        let filters = vec![
            FileNamed::wildmatch("*.image"),
            FileNamed::wildmatch("*.changes"),
            FileNamed::wildmatch("*.sources"),
        ]
        .into_iter()
        .map(|each| each.within(options.workspace()))
        .collect::<Vec<OneEntry>>();

        for ref filter in filters {
            zip_file(&mut zip, filter.as_path_buf()?, zip_options)?;
        }

        let gt_extra = options.workspace().join("gt-extra");
        zip_folder(&mut zip, &gt_extra, zip_options)?;

        for folder in options.gtoolkit_app_folders() {
            zip_folder(&mut zip, folder.as_path_buf()?, zip_options)?;
        }

        zip.finish()?;

        Ok(package)
    }
}
