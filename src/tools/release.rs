use crate::{Application, ExecutableSmalltalk, PlatformOS, Result, SmalltalkCommand};
use clap::Parser;
use feenk_releaser::VersionBump;
use file_matcher::FileNamed;
use std::path::{Path, PathBuf};
use zipper::ToZip;

#[derive(Parser, Debug, Clone)]
pub struct ReleaseOptions {
    /// Path to the .zip with the release image build. Supports mustache syntax to inject various release related information.
    /// The following properties are supported:
    /// - {{version}} - the release version in a form of X.Y.Z
    /// - {{os}} - the OS we release for. (`MacOS`, `Linux`, `Windows`)
    /// - {{arch}} - the target release architecture. (`x86_64`, `aarch64`)
    #[clap(parse(from_os_str))]
    pub release: PathBuf,
}

#[derive(Parser, Debug, Clone)]
pub struct ReleaserOptions {
    /// Specify a releaser version bump strategy
    #[clap(long, default_value = VersionBump::Patch.to_str(), possible_values = VersionBump::variants(), ignore_case = true)]
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

    fn process_template_path(application: &Application, path: impl AsRef<Path>) -> PathBuf {
        let new_version = application.image_version();

        let platform = match application.platform() {
            PlatformOS::MacOSX8664 | PlatformOS::MacOSAarch64 => "MacOS",
            PlatformOS::WindowsX8664 | PlatformOS::WindowsAarch64 => "Windows",
            PlatformOS::LinuxX8664 => "Linux",
        };

        let arch = match application.platform() {
            PlatformOS::MacOSX8664 => "x86_64",
            PlatformOS::MacOSAarch64 => "aarch64",
            PlatformOS::WindowsX8664 => "x86_64",
            PlatformOS::WindowsAarch64 => "aarch64",
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
        application: &Application,
        release_options: &ReleaseOptions,
    ) -> Result<PathBuf> {
        let package = Self::process_template_path(application, release_options.release.as_path());

        ToZip::new(package)
            .one_entry(FileNamed::wildmatch("*.image").within(application.workspace()))
            .one_entry(FileNamed::wildmatch("*.changes").within(application.workspace()))
            .one_entry(FileNamed::wildmatch("*.sources").within(application.workspace()))
            .folder(application.workspace().join("gt-extra"))
            .one_entries(application.gtoolkit_app_folders())
            .zip()
            .map_err(|error| error.into())
    }

    pub async fn run_releaser(
        &self,
        application: &Application,
        releaser_options: &ReleaserOptions,
    ) -> Result<()> {
        SmalltalkCommand::new("releasegtoolkit")
            .arg(format!("--strategy={}", releaser_options.bump.to_str()))
            .arg(format!(
                "--expected={}",
                application.image_version().to_string()
            ))
            .arg(if application.is_verbose() {
                "--verbose"
            } else {
                ""
            })
            .execute(&application.gtoolkit().evaluator())?;

        Ok(())
    }
}
