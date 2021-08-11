use crate::error::Error;
use crate::options::{AppOptions, PlatformOS};
use crate::{zip_file, zip_folder, ExecutableSmalltalk, SmalltalkCommand};
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
}

#[derive(Clap, Debug, Clone)]
#[clap(setting = AppSettings::ColorAlways)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct ReleaserOptions {
    /// Specify a releaser version bump strategy
    #[clap(long, default_value = VersionBump::Patch.to_str(), possible_values = VersionBump::variants(), case_insensitive = true)]
    pub bump: VersionBump,

    /// Public ssh key to use when pushing to the repositories
    #[clap(long, parse(from_os_str))]
    pub public_key: Option<PathBuf>,
    /// Private ssh key to use when pushing to the repositories
    #[clap(long, parse(from_os_str))]
    pub private_key: Option<PathBuf>,
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

    fn process_template_path(options: &AppOptions, path: impl AsRef<Path>) -> PathBuf {
        let new_version = options
            .gtoolkit_version()
            .expect("Must have a gtoolkit version");
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
        let package = Self::process_template_path(options, release_options.release.as_path());

        let file = std::fs::File::create(&package).unwrap();
        let mut zip = zip::ZipWriter::new(file);

        let zip_options =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        let filters = vec![
            FileNamed::wildmatch("*.image"),
            FileNamed::wildmatch("*.changes"),
            FileNamed::wildmatch("*.sources"),
            FileNamed::exact(options.vm_version_file_name()),
            FileNamed::exact(options.gtoolkit_version_file_name()),
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

    pub async fn run_releaser(
        &self,
        options: &AppOptions,
        releaser_options: &ReleaserOptions,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let private_key_path = if let Some(ref private_key) = releaser_options.private_key {
            if private_key.exists() {
                Some(to_absolute::canonicalize(private_key)?)
            } else {
                return Err(Box::new(Error {
                    what: format!("Specified private key does not exist: {:?}", private_key),
                    source: None,
                }));
            }
        } else {
            None
        };

        let public_key_path = if let Some(ref public_key) = releaser_options.public_key {
            if public_key.exists() {
                Some(to_absolute::canonicalize(public_key)?)
            } else {
                return Err(Box::new(Error {
                    what: format!("Specified public key does not exist: {:?}", public_key),
                    source: None,
                }));
            }
        } else {
            None
        };

        SmalltalkCommand::new("releasegtoolkit")
            .arg(format!("--strategy={}", releaser_options.bump.to_str()))
            .arg(options.gtoolkit_version().map_or_else(
                || "".to_string(),
                |version| format!("--expected={}", version.to_string()),
            ))
            .arg(private_key_path.map_or_else(
                || "".to_string(),
                |path| format!("--private-key={}", path.display()),
            ))
            .arg(public_key_path.map_or_else(
                || "".to_string(),
                |path| format!("--public-key={}", path.display()),
            ))
            .execute(&options.gtoolkit().evaluator())?;

        Ok(())
    }
}
