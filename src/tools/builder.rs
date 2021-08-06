use crate::create::FileToCreate;
use crate::download::{FileToDownload, FilesToDownload};
use crate::options::AppOptions;
use crate::{
    Checker, Downloader, FileToMove, SmalltalkScriptToExecute, SmalltalkScriptsToExecute, BUILDING,
    CREATING, DOWNLOADING, EXTRACTING, MOVING, SPARKLE,
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

#[derive(Clap, Debug, Clone)]
#[clap(setting = AppSettings::ColorAlways)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct ReleaseBuildOptions {
    #[clap(long, possible_values = Loader::VARIANTS, case_insensitive = true)]
    /// Specify a loader to install GToolkit code in a Pharo image.
    pub loader: Option<Loader>,
    /// Do not open a default GtWorld
    #[clap(long)]
    pub no_gt_world: bool,
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

        Checker::new()
            .check(options, build_options.should_overwrite())
            .await?;

        Downloader::new()
            .download_glamorous_toolkit_vm(options)
            .await?;

        println!("{}Downloading files...", DOWNLOADING);
        let pharo_image = FileToDownload::new(
            options.pharo_image_url(),
            options.workspace(),
            "pharo-image.zip",
        );

        let pharo_vm =
            FileToDownload::new(options.pharo_vm_url(), options.workspace(), "pharo-vm.zip");

        let files_to_download = FilesToDownload::new()
            .add(pharo_image.clone())
            .add(pharo_vm.clone());

        files_to_download.download().await?;

        println!("{}Extracting files...", EXTRACTING);

        let pharo_image_dir = options.workspace().join("pharo-image");

        let files_to_unzip = FilesToUnzip::new()
            .add(FileToUnzip::new(pharo_image.path(), &pharo_image_dir))
            .add(FileToUnzip::new(
                pharo_vm.path(),
                options.workspace().join("pharo-vm"),
            ));

        files_to_unzip.unzip().await?;

        println!("{}Moving files...", MOVING);

        FileToMove::new(
            FileNamed::wildmatch("*.image").within(&pharo_image_dir),
            options.workspace().join("GlamorousToolkit.image"),
        )
        .move_file()
        .await?;

        FileToMove::new(
            FileNamed::wildmatch("*.changes").within(&pharo_image_dir),
            options.workspace().join("GlamorousToolkit.changes"),
        )
        .move_file()
        .await?;

        FileToMove::new(
            FileNamed::wildmatch("*.sources").within(&pharo_image_dir),
            options.workspace(),
        )
        .move_file()
        .await?;

        let loader_st = match build_options.loader {
            Loader::Cloner => include_str!("../st/clone-gt.st"),
            Loader::Metacello => include_str!("../st/load-gt.st"),
        };

        println!("{}Creating build scripts...", CREATING);
        FileToCreate::new(
            options.workspace().join("load-patches.st"),
            include_str!("../st/load-patches.st"),
        )
        .create()
        .await?;
        FileToCreate::new(
            options.workspace().join("load-taskit.st"),
            include_str!("../st/load-taskit.st"),
        )
        .create()
        .await?;
        FileToCreate::new(options.workspace().join("load-gt.st"), loader_st)
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
