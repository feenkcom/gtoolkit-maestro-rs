use crate::create::FileToCreate;
use crate::download::{FileToDownload, FilesToDownload};
use crate::error::Error;
use crate::options::AppOptions;
use crate::{
    FileToMove, SmalltalkScriptToExecute, SmalltalkScriptsToExecute, BUILDING, CHECKING, CREATING,
    DOWNLOADING, EXTRACTING, MOVING, SPARKLE,
};
use crate::{FileToUnzip, FilesToUnzip};
use clap::{AppSettings, ArgEnum, Clap};
use file_matcher::FileNamed;
use indicatif::HumanDuration;
use std::str::FromStr;
use std::time::Instant;

#[derive(Clap, Debug, Clone)]
#[clap(setting = AppSettings::ColorAlways)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct BuildOptions {
    /// Delete existing installation of the gtoolkit if present
    #[clap(long)]
    pub overwrite: bool,
    #[clap(long, default_value = "cloner", possible_values = Loader::VARIANTS, case_insensitive = true)]
    /// Specify a loader to install GToolkit code in a Pharo image.
    pub loader: Loader,
}

impl BuildOptions {
    pub fn new() -> Self {
        Self {
            overwrite: false,
            loader: Loader::Cloner,
        }
    }
    pub fn should_overwrite(&self) -> bool {
        self.overwrite
    }

    pub fn overwrite(&mut self, overwrite: bool) {
        self.overwrite = overwrite;
    }

    pub fn loader(&mut self, loader: Loader) {
        self.loader = loader;
    }
}

#[derive(ArgEnum, Copy, Clone, Debug)]
#[repr(u32)]
pub enum Loader {
    /// Use Cloner from the https://github.com/feenkcom/gtoolkit-releaser, provides much faster loading speed but is not suitable for the release build
    #[clap(name = "cloner")]
    Cloner,
    /// Use Pharo's Metacello. Much slower than Cloner but is suitable for the release build
    #[clap(name = "metacello")]
    Metacello,
}

impl FromStr for Loader {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        <Loader as ArgEnum>::from_str(s, true)
    }
}

impl ToString for Loader {
    fn to_string(&self) -> String {
        (Loader::VARIANTS[*self as usize]).to_owned()
    }
}

pub struct Builder;

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
        if build_options.should_overwrite() && options.gtoolkit_directory().exists() {
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
            FileNamed::wildmatch("*.image").within(&pharo_image_dir),
            options.gtoolkit_directory().join("GlamorousToolkit.image"),
        )
        .move_file()
        .await?;

        FileToMove::new(
            FileNamed::wildmatch("*.changes").within(&pharo_image_dir),
            options
                .gtoolkit_directory()
                .join("GlamorousToolkit.changes"),
        )
        .move_file()
        .await?;

        FileToMove::new(
            FileNamed::wildmatch("*.sources").within(&pharo_image_dir),
            options.gtoolkit_directory(),
        )
        .move_file()
        .await?;

        let loader_st = match build_options.loader {
            Loader::Cloner => include_str!("../st/clone-gt.st"),
            Loader::Metacello => include_str!("../st/load-gt.st"),
        };

        println!("{}Creating build scripts...", CREATING);
        FileToCreate::new(
            options.gtoolkit_directory().join("load-patches.st"),
            include_str!("../st/load-patches.st"),
        )
        .create()
        .await?;
        FileToCreate::new(
            options.gtoolkit_directory().join("load-taskit.st"),
            include_str!("../st/load-taskit.st"),
        )
        .create()
        .await?;
        FileToCreate::new(options.gtoolkit_directory().join("load-gt.st"), loader_st)
            .create()
            .await?;

        let gtoolkit = options.gtoolkit();
        let pharo = options.pharo();

        println!("{}Preparing the image...", BUILDING);
        SmalltalkScriptsToExecute::new()
            .add(SmalltalkScriptToExecute::new("load-patches.st"))
            .add(SmalltalkScriptToExecute::new("load-taskit.st"))
            .execute(pharo.evaluator().save(true))
            .await?;

        println!("{}Building Glamorous Toolkit...", BUILDING);
        SmalltalkScriptsToExecute::new()
            .add(SmalltalkScriptToExecute::new("load-gt.st"))
            .execute(gtoolkit.evaluator().save(true))
            .await?;

        println!("{} Done in {}", SPARKLE, HumanDuration(started.elapsed()));
        Ok(())
    }
}
