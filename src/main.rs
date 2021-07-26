extern crate clap;
extern crate console;
extern crate feenk_releaser;
extern crate file_matcher;
extern crate octocrab as github;
extern crate regex;
extern crate reqwest;
extern crate semver;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate mustache;
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
    options.ensure_gtoolkit_version().await?;

    match options.command() {
        SubCommand::Build(build_options) => {
            Builder::new().build(&options, &build_options).await?;
        }
        SubCommand::Setup(setup_options) => {
            Setup::new().setup(&options, &setup_options).await?;
        }
        SubCommand::Test(test_options) => {
            Tester::new().test(&options, &test_options).await?;
        }
        SubCommand::LocalBuild => {
            let build_options = BuildOptions::new();
            let setup_options = SetupOptions::new();
            Builder::new().build(&options, &build_options).await?;
            Setup::new().setup(&options, &setup_options).await?;
        }
        SubCommand::ReleaseBuild(release_build) => {
            let mut build_options = BuildOptions::new();
            build_options.overwrite(true);

            if let Some(loader) = release_build.loader {
                build_options.loader(loader);
            } else {
                build_options.loader(Loader::Metacello);
            }

            let mut setup_options = SetupOptions::new();
            setup_options.target(SetupTarget::Release);

            Builder::new().build(&options, &build_options).await?;
            Setup::new().setup(&options, &setup_options).await?;
        }
        SubCommand::PackageTentative(tentative_options) => {
            Tentative::new()
                .package(&options, &tentative_options)
                .await?;
        }
        SubCommand::UnpackageTentative(tentative_options) => {
            Tentative::new()
                .unpackage(&mut options, &tentative_options)
                .await?;
        }
        SubCommand::PackageRelease(release_options) => {
            let package = Release::new().package(&options, &release_options).await?;
            println!("{}", package.display())
        }
        SubCommand::PrintDebug => {
            println!("{:?}", &options);
        }
    };

    Ok(())
}
