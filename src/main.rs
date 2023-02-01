#[macro_use]
extern crate serde_derive;

mod application;
mod create;
mod error;
mod gtoolkit;
mod moving;
mod options;
mod seed;
mod smalltalk;
mod tools;
mod version;

pub use application::*;
pub use error::*;
pub use gtoolkit::*;
pub use moving::*;
pub use seed::*;
pub use smalltalk::*;
pub use tools::*;
pub use version::*;

use crate::options::SubCommand;
use clap::Parser;
use options::AppOptions;
use url::Url;
use user_error::{UserFacingError, UFE};

pub const DEFAULT_IMAGE_NAME: &str = "GlamorousToolkit";
pub const DEFAULT_IMAGE_EXTENSION: &str = "image";

pub const DEFAULT_PHARO_VM_MAC_X86_64: &str =
    "https://dl.feenk.com/pharo/PharoVM-9.0.15-b4879008-Darwin-x86_64-bin.zip";
pub const DEFAULT_PHARO_VM_MAC_AARCH64: &str =
    "https://dl.feenk.com/pharo/PharoVM-9.0.15-b4879008-Darwin-arm64-bin.zip";
pub const DEFAULT_PHARO_VM_LINUX_X86_64: &str =
    "https://dl.feenk.com/pharo/PharoVM-9.0.15-b487900-Linux-x86_64-bin.zip";
pub const DEFAULT_PHARO_VM_LINUX_AARCH64: &str =
    "https://dl.feenk.com/pharo/PharoVM-9.0.15-b487900-Linux-aarch64-bin.zip";
pub const DEFAULT_PHARO_VM_WINDOWS: &str =
    "https://dl.feenk.com/pharo/PharoVM-9.0.15-b4879008-Windows-x86_64-bin.zip";

pub const DEFAULT_PHARO_IMAGE: &str =
    "https://dl.feenk.com/pharo/Pharo10-SNAPSHOT.build.521.sha.14f5413.arch.64bit.zip";

pub const SERIALIZATION_FILE: &str = "gtoolkit.yaml";

pub const GTOOLKIT_REPOSITORY_OWNER: &str = "feenkcom";
pub const GTOOLKIT_REPOSITORY_NAME: &str = "gtoolkit";

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
            let deserialized_application = application.deserialize_application_from_file()?;
            println!("v{}", &deserialized_application.image_version());
        }
        SubCommand::PrintGtoolkitAppVersion => {
            let deserialized_application = application.deserialize_application_from_file()?;
            println!("v{}", &deserialized_application.app_version());
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
