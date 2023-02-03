use std::path::PathBuf;

use clap::Parser;

use crate::LocalBuildOptions;
use crate::{
    BuildOptions, CopyOptions, ReleaseBuildOptions, ReleaseOptions, ReleaserOptions, RenameOptions,
    SetupOptions, StartOptions, TentativeOptions, TestOptions,
};

pub const DEFAULT_DIRECTORY: &str = "glamoroustoolkit";

pub const VM_REPOSITORY_OWNER: &str = "feenkcom";
pub const VM_REPOSITORY_NAME: &str = "gtoolkit-vm";

#[derive(Parser, Clone, Debug)]
#[clap(author)]
pub struct AppOptions {
    #[clap(subcommand)]
    sub_command: SubCommand,
    /// Perform commands in a verbose manner
    #[clap(long)]
    verbose: bool,
    #[clap(long, default_value = DEFAULT_DIRECTORY, parse(from_os_str))]
    workspace: PathBuf,
}

#[derive(Parser, Clone, Debug)]
pub enum SubCommand {
    /// Creates a typical local build of GlamorousToolkit with GtWorld opened and sets the image up. This is intended to be used by developers and contributors.
    #[clap(display_order = 1)]
    LocalBuild(LocalBuildOptions),
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
    /// Rename the image deleting the old one
    #[clap(display_order = 6)]
    RenameTo(RenameOptions),
    /// Starts an application interactively, waits for a duration of delay to let it load completely then saves and quits.
    #[clap(display_order = 7)]
    Start(StartOptions),
    /// Cleans up an image after loading Glamorous Toolkit. It cleans up ssh keys, removes iceberg repositories
    /// and garbage collects objects
    #[clap(display_order = 8)]
    CleanUp,
    /// Tests Glamorous Toolkit and exports the results.
    #[clap(display_order = 9)]
    Test(TestOptions),
    /// Package the GlamorousToolkit image as a tentative release.
    #[clap(display_order = 10)]
    PackageTentative(TentativeOptions),
    /// Given a packaged tentative image, download the GlamorousToolkit app for the version specified in the .version file
    #[clap(display_order = 11)]
    UnpackageTentative(TentativeOptions),
    /// Package the GlamorousToolkit image and App for a release. Prints the path to the created package in the `stdout`
    #[clap(display_order = 12)]
    PackageRelease(ReleaseOptions),
    /// Run the gtoolkit-releaser to release glamorous toolkit
    #[clap(display_order = 13)]
    RunReleaser(ReleaserOptions),
    /// Display the Debug information of the AppOptions
    #[clap(display_order = 14)]
    PrintDebug,
    /// Display the version of the glamorous toolkit image from the .yaml file in the workspace.
    /// Fails if the .yaml file wasn't found.
    #[clap(display_order = 15)]
    PrintGtoolkitImageVersion,
    /// Display the version of the glamorous toolkit app from the .yaml file in the workspace.
    /// Fails if the .yaml file wasn't found.
    #[clap(display_order = 16)]
    PrintGtoolkitAppVersion,
}

impl AppOptions {
    pub fn command(&self) -> SubCommand {
        self.sub_command.clone()
    }

    pub fn workspace(&self) -> PathBuf {
        let workspace = self.workspace.as_path();
        if workspace.is_relative() {
            std::env::current_dir().unwrap().join(workspace)
        } else {
            workspace.to_path_buf()
        }
    }

    pub fn verbose(&self) -> bool {
        self.verbose
    }
}
