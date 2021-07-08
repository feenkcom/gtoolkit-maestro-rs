use clap::{AppSettings, Clap};
use std::path::PathBuf;

pub const DEFAULT_REPOSITORY: &str = "https://github.com/feenkcom/gtoolkit.git";
pub const DEFAULT_BRANCH: &str = "main";
pub const DEFAULT_DIRECTORY: &str = "glamoroustoolkit";

#[derive(Clap, Clone, Debug)]
#[clap(author = "feenk gmbh <contact@feenk.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct BuildOptions {
    /// A name of the environment variable with personal GitHub token. The reason we do not accept tokens directly is because then it would be exposed in the CI log
    #[clap(long)]
    token: Option<String>,
    #[clap(subcommand)]
    sub_command: SubCommand,
}

#[derive(Clap, Clone, Debug)]
pub enum SubCommand {
    Build(Build),
    Get,
    Clone,
}

/// Builds GlamorousToolkit from sources
#[derive(Clap, Debug, Clone)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Build {
    /// Delete existing installation of the gtoolkit if present
    #[clap(long)]
    overwrite: bool,
}

impl BuildOptions {
    pub fn command(&self) -> SubCommand {
        self.sub_command.clone()
    }

    pub fn repository(&self) -> String {
        DEFAULT_REPOSITORY.to_owned()
    }

    pub fn branch(&self) -> String {
        DEFAULT_BRANCH.to_owned()
    }

    pub fn gtoolkit_directory(&self) -> PathBuf {
        std::env::current_dir().unwrap().join(DEFAULT_DIRECTORY)
    }

    pub fn should_overwrite(&self) -> bool {
        match &self.sub_command {
            SubCommand::Build(build) => build.overwrite,
            SubCommand::Get => false,
            SubCommand::Clone => false,
        }
    }

    pub fn pharo_executable(&self) -> PathBuf {
        self.gtoolkit_directory()
            .join("pharo-vm")
            .join("Pharo.app")
            .join("Contents")
            .join("MacOS")
            .join("Pharo")
    }

    pub fn gtoolkit_executable(&self) -> PathBuf {
        self.gtoolkit_directory()
            .join("GlamorousToolkit.app")
            .join("Contents")
            .join("MacOS")
            .join("GlamorousToolkit-cli")
    }

    pub fn gtoolkit_image(&self) -> PathBuf {
        self.gtoolkit_directory().join("GlamorousToolkit.image")
    }
}
