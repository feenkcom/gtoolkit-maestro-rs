extern crate clap;
extern crate console;
extern crate feenk_releaser;
extern crate file_matcher;
extern crate octocrab as github;
extern crate regex;
extern crate reqwest;
extern crate semver;
extern crate serde;
extern crate serde_derive;
extern crate tokio;
extern crate tokio_stream;
extern crate tokio_util;
extern crate walkdir;
extern crate zip;

mod create;
mod download;
mod error;
mod gtoolkit;
mod moving;
mod options;
mod smalltalk;
mod tools;
mod zipping;

pub use moving::*;
pub use smalltalk::*;
pub use tools::*;
pub use zipping::*;

use crate::options::SubCommand;
use clap::Clap;
use options::AppOptions;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut options: AppOptions = AppOptions::parse();
    options.ensure_vm_version().await?;

    match options.command() {
        SubCommand::Build(build_options) => {
            Builder::new().build(&options, &build_options).await?;
        }
        SubCommand::Setup(setup_options) => {
            Setup::new().setup(&options, &setup_options).await?;
        }
        SubCommand::LocalBuild => {
            let build_options = BuildOptions::new();
            let setup_options = SetupOptions::new();
            Builder::new().build(&options, &build_options).await?;
            Setup::new().setup(&options, &setup_options).await?;
        }
        SubCommand::ReleaseBuild => {
            let mut build_options = BuildOptions::new();
            build_options.overwrite(true);
            build_options.loader(Loader::Metacello);

            let mut setup_options = SetupOptions::new();
            setup_options.target(SetupTarget::Release);

            Builder::new().build(&options, &build_options).await?;
            Setup::new().setup(&options, &setup_options).await?;
        }
        SubCommand::PackageImage => {
            Packager::new().package_image(&options).await?;
        }
    };

    Ok(())
}
