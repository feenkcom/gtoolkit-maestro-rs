use crate::create::FileToCreate;
use crate::download::{FileToDownload, FilesToDownload};
use crate::error::Error;
use crate::options::{AppOptions, BuildOptions, Loader};
use crate::smalltalking::SmalltalkScriptToExecute;
use crate::smalltalking::SmalltalkScriptsToExecute;
use crate::unzip::{FileToUnzip, FilesToUnzip};
use crate::FileToMove;
use console::Emoji;
use indicatif::HumanDuration;
use std::time::Instant;

pub struct Builder;

static CHECKING: Emoji<'_, '_> = Emoji("🔍 ", "");
static DOWNLOADING: Emoji<'_, '_> = Emoji("📥 ", "");
static EXTRACTING: Emoji<'_, '_> = Emoji("📦 ", "");
static MOVING: Emoji<'_, '_> = Emoji("🚚 ", "");
static CREATING: Emoji<'_, '_> = Emoji("📝 ", "");
static BUILDING: Emoji<'_, '_> = Emoji("🏗️  ", "");
static SPARKLE: Emoji<'_, '_> = Emoji("✨ ", ":-)");

impl Builder {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn build(
        &self,
        options: &AppOptions,
        build_options: &BuildOptions,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let started = Instant::now();

        println!("{}Checking the system...", CHECKING);
        if options.should_overwrite() && options.gtoolkit_directory().exists() {
            tokio::fs::remove_dir_all(options.gtoolkit_directory()).await?;
        }

        if options.gtoolkit_directory().exists() {
            return Err(Box::new(Error {
                what: format!(
                    "GToolkit already exists in {:?}",
                    options.gtoolkit_directory().display()
                ),
                source: None,
            }));
        }

        tokio::fs::create_dir_all(options.gtoolkit_directory()).await?;

        println!("{}Downloading files...", DOWNLOADING);
        let pharo_image = FileToDownload::new(
            options.pharo_image_url(),
            options.gtoolkit_directory(),
            "pharo-image.zip",
        );

        let pharo_vm = FileToDownload::new(
            options.pharo_vm_url(),
            options.gtoolkit_directory(),
            "pharo-vm.zip",
        );

        let gtoolkit_vm = FileToDownload::new(
            options.gtoolkit_app_url(),
            options.gtoolkit_directory(),
            "GlamorousToolkit.zip",
        );

        let files_to_download = FilesToDownload::new()
            .add(pharo_image.clone())
            .add(pharo_vm.clone())
            .add(gtoolkit_vm.clone());

        files_to_download.download().await?;

        println!("{}Extracting files...", EXTRACTING);

        let pharo_image_dir = options.gtoolkit_directory().join("pharo-image");

        let files_to_unzip = FilesToUnzip::new()
            .add(FileToUnzip::new(pharo_image.path(), &pharo_image_dir))
            .add(FileToUnzip::new(
                pharo_vm.path(),
                options.gtoolkit_directory().join("pharo-vm"),
            ))
            .add(FileToUnzip::new(
                gtoolkit_vm.path(),
                options.gtoolkit_directory(),
            ));

        files_to_unzip.unzip().await?;

        println!("{}Moving files...", MOVING);

        FileToMove::new(
            ".*image",
            &pharo_image_dir,
            options.gtoolkit_directory().join("GlamorousToolkit.image"),
        )
        .move_file()
        .await?;

        FileToMove::new(
            ".*changes",
            &pharo_image_dir,
            options
                .gtoolkit_directory()
                .join("GlamorousToolkit.changes"),
        )
        .move_file()
        .await?;

        let loader_st = match build_options.loader {
            Loader::Cloner => include_str!("st/clone-gt.st"),
            Loader::Metacello => include_str!("st/load-gt.st"),
        };

        FileToMove::new(".*sources", &pharo_image_dir, options.gtoolkit_directory())
            .move_file()
            .await?;

        println!("{}Creating build scripts...", CREATING);
        FileToCreate::new(
            options.gtoolkit_directory().join("load-patches.st"),
            include_str!("st/load-patches.st"),
        )
        .create()
        .await?;
        FileToCreate::new(
            options.gtoolkit_directory().join("load-taskit.st"),
            include_str!("st/load-taskit.st"),
        )
        .create()
        .await?;
        FileToCreate::new(options.gtoolkit_directory().join("loader.st"), loader_st)
            .create()
            .await?;
        FileToCreate::new(
            options.gtoolkit_directory().join("start-gt.st"),
            include_str!("st/start-gt.st"),
        )
        .create()
        .await?;

        println!("{}Building the image...", BUILDING);
        SmalltalkScriptsToExecute::new(options.gtoolkit_directory())
            .add(SmalltalkScriptToExecute::new(
                options.pharo_executable(),
                options.gtoolkit_image(),
                "load-patches.st",
            ))
            .add(SmalltalkScriptToExecute::new(
                options.pharo_executable(),
                options.gtoolkit_image(),
                "load-taskit.st",
            ))
            .add(SmalltalkScriptToExecute::new(
                options.gtoolkit_app_cli(),
                options.gtoolkit_image(),
                "loader.st",
            ))
            .add(
                SmalltalkScriptToExecute::new(
                    options.gtoolkit_app_cli(),
                    options.gtoolkit_image(),
                    "start-gt.st",
                )
                .no_quit()
                .interactive(),
            )
            .execute()
            .await?;

        println!("{} Done in {}", SPARKLE, HumanDuration(started.elapsed()));
        println!("To start GlamorousToolkit run:");
        println!("  cd {:?}", options.gtoolkit_directory());
        println!("  {}", options.gtoolkit_app());
        Ok(())
    }
}
