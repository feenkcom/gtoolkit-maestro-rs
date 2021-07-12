use crate::create::FileToCreate;
use crate::download::{FileToDownload, FilesToDownload};
use crate::error::Error;
use crate::moving::FileToMove;
use crate::options::{AppOptions, PlatformOS};
use crate::smalltalking::SmalltalkScriptToExecute;
use crate::smalltalking::SmalltalkScriptsToExecute;
use crate::unzip::{FileToUnzip, FilesToUnzip};
use console::Emoji;
use indicatif::HumanDuration;
use std::process::Command;
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

    pub fn gtoolkit_app_url(&self, options: &AppOptions) -> &str {
        match options.platform() {
            PlatformOS::MacOSX8664 => {
                "https://github.com/feenkcom/gtoolkit-vm/releases/latest/download/GlamorousToolkit-x86_64-apple-darwin.app.zip"
            }
            PlatformOS::MacOSAarch64 => {
                "https://github.com/feenkcom/gtoolkit-vm/releases/latest/download/GlamorousToolkit-aarch64-apple-darwin.app.zip"
            }
            PlatformOS::WindowsX8664 => {
                "https://github.com/feenkcom/gtoolkit-vm/releases/latest/download/GlamorousToolkit-x86_64-pc-windows-msvc.zip"
            }
            PlatformOS::LinuxX8664 => {
                "https://github.com/feenkcom/gtoolkit-vm/releases/latest/download/GlamorousToolkit-x86_64-unknown-linux-gnu.zip"
            }
        }
    }

    pub fn gtoolkit_app_cli(&self, options: &AppOptions) -> &str {
        match options.platform() {
            PlatformOS::MacOSX8664 => { "GlamorousToolkit.app/Contents/MacOS/GlamorousToolkit-cli" }
            PlatformOS::MacOSAarch64 => { "GlamorousToolkit.app/Contents/MacOS/GlamorousToolkit-cli" }
            PlatformOS::WindowsX8664 => { "GlamorousToolkit/bin/GlamorousToolkit-cli.exe" }
            PlatformOS::LinuxX8664 => { "GlamorousToolkit/bin/GlamorousToolkit-cli" }
        }
    }

    pub fn gtoolkit_app(&self, options: &AppOptions) -> &str {
        match options.platform() {
            PlatformOS::MacOSX8664 => { "GlamorousToolkit.app/Contents/MacOS/GlamorousToolkit" }
            PlatformOS::MacOSAarch64 => { "GlamorousToolkit.app/Contents/MacOS/GlamorousToolkit" }
            PlatformOS::WindowsX8664 => { "bin\\GlamorousToolkit.exe" }
            PlatformOS::LinuxX8664 => { "./bin/GlamorousToolkit" }
        }
    }

    pub async fn build(&self, options: &AppOptions) -> Result<(), Box<dyn std::error::Error>> {
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
            self.gtoolkit_app_url(options),
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

        FileToMove::new(".*sources", &pharo_image_dir, options.gtoolkit_directory())
            .move_file()
            .await?;

        Command::new(self.gtoolkit_app_cli(options))
            .current_dir(options.gtoolkit_directory())
            .arg("GlamorousToolkit.image")
            .arg("st")
            .arg("-quit")
            .arg("");

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
        FileToCreate::new(
            options.gtoolkit_directory().join("clone-gt.st"),
            include_str!("st/clone-gt.st"),
        )
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
                options.gtoolkit_executable(),
                options.gtoolkit_image(),
                "clone-gt.st",
            ))
            .add(
                SmalltalkScriptToExecute::new(
                    options.gtoolkit_executable(),
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
        println!("  {}", self.gtoolkit_app(options));
        Ok(())
    }
}
