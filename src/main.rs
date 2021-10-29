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
extern crate parse_duration;
extern crate thiserror;
extern crate to_absolute;
extern crate tokio;
extern crate tokio_stream;
extern crate tokio_util;
extern crate user_error;
extern crate walkdir;
extern crate zip;

mod application;
mod create;
mod download;
mod error;
mod gtoolkit;
mod moving;
mod options;
mod seed;
mod smalltalk;
mod tools;
mod version;
mod zipping;

pub use application::*;
pub use download::*;
pub use error::*;
pub use gtoolkit::*;
pub use moving::*;
pub use seed::*;
pub use smalltalk::*;
pub use tools::*;
pub use version::*;
pub use zipping::*;

use crate::options::SubCommand;
use clap::Clap;
use options::AppOptions;
use url::Url;
use user_error::{UserFacingError, UFE};

async fn run() -> Result<()> {
    let options: AppOptions = AppOptions::parse();

    let gtoolkit_vm_version = options.fetch_vm_version().await?;
    let gtoolkit_image_version = Application::latest_gtoolkit_image_version().await?;
    let image_seed = ImageSeed::Url(Url::parse(DEFAULT_PHARO_IMAGE)?);

    let mut application = Application::new(
        options.workspace(),
        gtoolkit_vm_version,
        gtoolkit_image_version,
        image_seed,
    )?;
    application.set_verbose(options.verbose());

    if application.serialization_file().exists() {
        application.deserialize_from_file()?;
    }

    match options.command() {
        SubCommand::Build(build_options) => {
            Builder::new()
                .build(&mut application, &build_options)
                .await?;
        }
        SubCommand::Setup(setup_options) => {
            Setup::new().setup(&mut application, &setup_options).await?;
        }
        SubCommand::Test(test_options) => {
            Tester::new().test(&application, &test_options).await?;
        }
        SubCommand::LocalBuild(local_build) => {
            let mut setup_options = SetupOptions::new();
            setup_options.target(SetupTarget::LocalBuild);
            setup_options.gt_world(!local_build.no_gt_world);

            Builder::new()
                .build(&mut application, &local_build.build_options)
                .await?;
            Setup::new().setup(&mut application, &setup_options).await?;
        }
        SubCommand::ReleaseBuild(release_build) => {
            let mut setup_options = SetupOptions::new();
            setup_options.target(SetupTarget::Release);
            setup_options.gt_world(!release_build.no_gt_world);
            setup_options.bump(release_build.bump);

            Builder::new()
                .build(&mut application, &release_build.build_options)
                .await?;
            Setup::new().setup(&mut application, &setup_options).await?;
        }
        SubCommand::CopyTo(copy_options) => {
            Copier::new().copy(&mut application, &copy_options).await?;
        }
        SubCommand::RenameTo(rename_options) => {
            Renamer::new()
                .rename(&mut application, &rename_options)
                .await?;
        }
        SubCommand::CleanUp => {
            Cleaner::new().clean(&application).await?;
        }
        SubCommand::Start(start_options) => {
            Starter::new().start(&application, &start_options).await?;
        }
        SubCommand::PackageTentative(tentative_options) => {
            Tentative::new()
                .package(&application, &tentative_options)
                .await?;
        }
        SubCommand::UnpackageTentative(tentative_options) => {
            Tentative::new()
                .unpackage(&mut application, &tentative_options)
                .await?;
        }
        SubCommand::PackageRelease(release_options) => {
            let package = Release::new()
                .package(&application, &release_options)
                .await?;
            println!("{}", package.display())
        }
        SubCommand::RunReleaser(releaser_options) => {
            Release::new()
                .run_releaser(&application, &releaser_options)
                .await?;
        }
        SubCommand::PrintDebug => {
            println!("{:?}", &application);
        }
        SubCommand::PrintGtoolkitImageVersion => {
            println!("v{}", &application.image_version());
        }
        SubCommand::PrintGtoolkitAppVersion => {
            println!("v{}", &application.app_version());
        }
    };

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        let error: Box<dyn std::error::Error> = Box::new(error);
        let user_facing_error: UserFacingError = error.into();
        user_facing_error.help("").print_and_exit();
    }
}
