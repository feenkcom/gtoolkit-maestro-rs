use crate::Result;
use crate::{
    AppVersion, BuildOptions, CopyOptions, ImageVersion, InstallerError, ReleaseBuildOptions,
    ReleaseOptions, ReleaserOptions, SetupOptions, StartOptions, TentativeOptions, TestOptions,
};
use clap::{AppSettings, Clap};
use feenk_releaser::{GitHub, Version};
use std::path::PathBuf;

pub const DEFAULT_DIRECTORY: &str = "glamoroustoolkit";

pub const GTOOLKIT_REPOSITORY_OWNER: &str = "feenkcom";
pub const GTOOLKIT_REPOSITORY_NAME: &str = "gtoolkit";

pub const VM_REPOSITORY_OWNER: &str = "feenkcom";
pub const VM_REPOSITORY_NAME: &str = "gtoolkit-vm";

#[derive(Clap, Clone, Debug)]
#[clap(author = "feenk gmbh <contact@feenk.com>")]
#[clap(setting = AppSettings::ColorAlways)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct AppOptions {
    #[clap(subcommand)]
    sub_command: SubCommand,
    /// Perform commands in a verbose manner
    #[clap(long)]
    verbose: bool,
    #[clap(long, default_value = DEFAULT_DIRECTORY, parse(from_os_str))]
    workspace: PathBuf,
}

#[derive(Clap, Clone, Debug)]
pub enum SubCommand {
    /// Creates a typical local build of GlamorousToolkit with GtWorld opened and sets the image up. This is intended to be used by developers and contributors.
    #[clap(display_order = 1)]
    LocalBuild,
    /// Creates a release build of GlamorousToolkit with GtWorld opened and sets up the image to be deployed. This is intended to be used by the Continuous Integration server.
    #[clap(display_order = 2)]
    ReleaseBuild(ReleaseBuildOptions),
    /// Builds GlamorousToolkit image from sources without performing any extra setup.
    #[clap(display_order = 3)]
    Build(BuildOptions),
    /// Sets up the GlamorousToolkit image. This includes opening a default GtWorld and configuring various settings.
    #[clap(display_order = 4)]
    Setup(SetupOptions),
    /// Copies glamorous toolkit related files from the current workspace into a new workspace.
    /// Does not copy temporary files or logs
    #[clap(display_order = 5)]
    CopyTo(CopyOptions),
    /// Starts an application interactively, waits for a duration of delay to let it load completely then saves and quits.
    #[clap(display_order = 6)]
    Start(StartOptions),
    /// Cleans up an image after loading Glamorous Toolkit. It cleans up ssh keys, removes iceberg repositories
    /// and garbage collects objects
    #[clap(display_order = 7)]
    CleanUp,
    /// Tests Glamorous Toolkit and exports the results.
    #[clap(display_order = 8)]
    Test(TestOptions),
    /// Package the GlamorousToolkit image as a tentative release.
    #[clap(display_order = 9)]
    PackageTentative(TentativeOptions),
    /// Given a packaged tentative image, download the GlamorousToolkit app for the version specified in the .version file
    #[clap(display_order = 10)]
    UnpackageTentative(TentativeOptions),
    /// Package the GlamorousToolkit image and App for a release. Prints the path to the created package in the `stdout`
    #[clap(display_order = 11)]
    PackageRelease(ReleaseOptions),
    /// Run the gtoolkit-releaser to release glamorous toolkit
    #[clap(display_order = 12)]
    RunReleaser(ReleaserOptions),
    /// Display the Debug information of the AppOptions
    #[clap(display_order = 13)]
    PrintDebug,
    /// Display the version of the glamorous toolkit
    #[clap(display_order = 14)]
    PrintGtoolkitImageVersion,
    /// Display the version of the glamorous toolkit app
    #[clap(display_order = 15)]
    PrintGtoolkitAppVersion,
}

impl AppOptions {
    pub fn command(&self) -> SubCommand {
        self.sub_command.clone()
    }

    pub async fn fetch_vm_version(&self) -> Result<AppVersion> {
        let latest_version: Option<Version> =
            GitHub::new(VM_REPOSITORY_OWNER, VM_REPOSITORY_NAME, None)
                .latest_release_version()
                .await?;

        if let Some(latest_version) = latest_version {
            return Ok(latest_version.into());
        };

        InstallerError::FailedToDetectGlamorousAppVersion.into()
    }

    pub async fn fetch_image_version(&self) -> Result<ImageVersion> {
        let latest_version: Option<Version> =
            GitHub::new(GTOOLKIT_REPOSITORY_OWNER, GTOOLKIT_REPOSITORY_NAME, None)
                .latest_release_version()
                .await?;

        if let Some(latest_version) = latest_version {
            return Ok(latest_version.into());
        };

        InstallerError::FailedToDetectGlamorousImageVersion.into()
    }

    pub fn workspace(&self) -> PathBuf {
        std::env::current_dir()
            .unwrap()
            .join(self.workspace.as_path())
    }

    pub fn verbose(&self) -> bool {
        self.verbose
    }
}
