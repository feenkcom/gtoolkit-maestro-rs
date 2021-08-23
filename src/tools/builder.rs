use crate::create::FileToCreate;
use crate::download::{FileToDownload, FilesToDownload};
use crate::{
    Application, Checker, Downloader, ExecutableSmalltalk, FileToMove, ImageSeed, InstallerError,
    Result, Smalltalk, SmalltalkCommand, SmalltalkExpressionBuilder, SmalltalkScriptToExecute,
    SmalltalkScriptsToExecute, BUILDING, CREATING, DOWNLOADING, EXTRACTING, MOVING, SPARKLE,
};
use crate::{FileToUnzip, FilesToUnzip};
use clap::{AppSettings, ArgEnum, Clap};
use feenk_releaser::VersionBump;
use file_matcher::FileNamed;
use indicatif::HumanDuration;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;
use url::Url;

pub const DEFAULT_PHARO_IMAGE: &str =
    "https://dl.feenk.com/pharo/Pharo9.0-SNAPSHOT.build.1532.sha.e58ef49.arch.64bit.zip";

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
    /// Specify a URL to a clean seed image on top of which to build the glamorous toolkit
    #[clap(long, parse(try_from_str = url_parse), conflicts_with_all(&["image_zip", "image_file"]))]
    pub image_url: Option<Url>,
    /// Specify a path to the zip archive that contains a seed .image, .changes and .sources on top of which to build the glamorous toolkit
    #[clap(long, parse(from_os_str), conflicts_with_all(&["image_url", "image_file"]))]
    pub image_zip: Option<PathBuf>,
    /// Specify a path to the .image in which to install the glamorous toolkit
    #[clap(long, parse(from_os_str), conflicts_with_all(&["image_url", "image_zip"]))]
    pub image_file: Option<PathBuf>,
    /// Public ssh key to use when cloning repositories
    #[clap(long, parse(from_os_str))]
    pub public_key: Option<PathBuf>,
    /// Private ssh key to use when cloning repositories
    #[clap(long, parse(from_os_str))]
    pub private_key: Option<PathBuf>,
}

impl BuildOptions {
    pub fn image_seed(&self) -> ImageSeed {
        if let Some(ref image_zip) = self.image_zip {
            return ImageSeed::Zip(image_zip.clone());
        }
        if let Some(ref image_url) = self.image_url {
            return ImageSeed::Url(image_url.clone());
        }

        if let Some(ref image_file) = self.image_file {
            return ImageSeed::Image(image_file.clone());
        }

        return ImageSeed::Url(
            url_parse(DEFAULT_PHARO_IMAGE)
                .unwrap_or_else(|_| panic!("Failed to parse url: {}", DEFAULT_PHARO_IMAGE)),
        );
    }
}

fn url_parse(val: &str) -> Result<Url> {
    Url::parse(val).map_err(|error| error.into())
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
    /// When building an image for a release, specify which component version to bump
    #[clap(long, default_value = VersionBump::Patch.to_str(), possible_values = VersionBump::variants(), case_insensitive = true)]
    pub bump: VersionBump,
    /// Public ssh key to use when pushing to the repositories
    #[clap(long, parse(from_os_str))]
    pub public_key: Option<PathBuf>,
    /// Private ssh key to use when pushing to the repositories
    #[clap(long, parse(from_os_str))]
    pub private_key: Option<PathBuf>,
}

impl BuildOptions {
    fn ssh_keys(&self) -> Result<Option<(PathBuf, PathBuf)>> {
        let public_key = self.public_key()?;
        let private_key = self.private_key()?;

        match (&private_key, &public_key) {
            (Some(private), Some(public)) => Ok(Some((private.clone(), public.clone()))),
            (None, None) => Ok(None),
            _ => InstallerError::SshKeysConfigurationError(private_key, public_key).into(),
        }
    }

    fn public_key(&self) -> Result<Option<PathBuf>> {
        if let Some(ref key) = self.public_key {
            if key.exists() {
                Ok(Some(to_absolute::canonicalize(key).map_err(|error| {
                    InstallerError::CanonicalizeError(key.clone(), error)
                })?))
            } else {
                return InstallerError::PublicKeyDoesNotExist(key.clone()).into();
            }
        } else {
            Ok(None)
        }
    }

    fn private_key(&self) -> Result<Option<PathBuf>> {
        if let Some(ref key) = self.private_key {
            if key.exists() {
                Ok(Some(to_absolute::canonicalize(key).map_err(|error| {
                    InstallerError::CanonicalizeError(key.clone(), error)
                })?))
            } else {
                return InstallerError::PrivateKeyDoesNotExist(key.clone()).into();
            }
        } else {
            Ok(None)
        }
    }
}

impl BuildOptions {
    pub fn new() -> Self {
        Self {
            overwrite: false,
            loader: Loader::Cloner,
            image_url: None,
            image_zip: None,
            image_file: None,
            public_key: None,
            private_key: None,
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

    fn from_str(s: &str) -> std::result::Result<Self, String> {
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
        application: &mut Application,
        build_options: &BuildOptions,
    ) -> Result<()> {
        let started = Instant::now();

        let image_seed = build_options.image_seed();
        application.set_image_seed(image_seed.clone())?;

        Checker::new()
            .check(application, build_options.should_overwrite())
            .await?;

        application.serialize_into_file()?;

        println!("{}Downloading files...", DOWNLOADING);

        let pharo_vm = FileToDownload::new(
            Url::parse(application.pharo_vm_url())?,
            application.workspace(),
            "pharo-vm.zip",
        );

        let files_to_download = FilesToDownload::new()
            .extend(Downloader::files_to_download(application))
            .add(pharo_vm.clone())
            .maybe_add(image_seed.file_to_download(application));

        files_to_download.download().await?;

        println!("{}Extracting files...", EXTRACTING);

        let files_to_unzip = FilesToUnzip::new()
            .extend(Downloader::files_to_unzip(application))
            .add(FileToUnzip::new(
                pharo_vm.path(),
                application.workspace().join("pharo-vm"),
            ))
            .maybe_add(image_seed.file_to_unzip(application));

        files_to_unzip.unzip().await?;

        if !image_seed.is_image_file() {
            println!("{}Moving files...", MOVING);

            let seed_image = FileNamed::wildmatch(format!("*.{}", application.image_extension()))
                .within(image_seed.seed_image_directory(application))
                .find()?;
            let seed_smalltalk =
                Smalltalk::new(application.pharo_executable(), seed_image, application);
            let seed_evaluator = seed_smalltalk.evaluator();

            SmalltalkCommand::new("save")
                .arg(
                    application
                        .workspace()
                        .join(application.image_name())
                        .display()
                        .to_string(),
                )
                .execute(&seed_evaluator)?;

            FileToMove::new(
                FileNamed::wildmatch("*.sources")
                    .within(image_seed.seed_image_directory(application))
                    .find()?,
                application.workspace(),
            )
            .move_file()
            .await?;
        }

        let loader_st = match build_options.loader {
            Loader::Cloner => include_str!("../st/clone-gt.st"),
            Loader::Metacello => include_str!("../st/load-gt.st"),
        };

        println!("{}Creating build scripts...", CREATING);
        FileToCreate::new(
            application.workspace().join("load-patches.st"),
            include_str!("../st/load-patches.st"),
        )
        .create()
        .await?;
        FileToCreate::new(
            application.workspace().join("load-taskit.st"),
            include_str!("../st/load-taskit.st"),
        )
        .create()
        .await?;
        FileToCreate::new(application.workspace().join("load-gt.st"), loader_st)
            .create()
            .await?;

        let gtoolkit = application.gtoolkit();
        let pharo = application.pharo();

        println!("{}Preparing the image...", BUILDING);
        SmalltalkScriptsToExecute::new()
            .add(SmalltalkScriptToExecute::new("load-patches.st"))
            .add(SmalltalkScriptToExecute::new("load-taskit.st"))
            .execute(pharo.evaluator().save(true))
            .await?;

        println!("{}Building Glamorous Toolkit...", BUILDING);
        let ssh_keys = build_options.ssh_keys()?;
        let mut scripts_to_execute = SmalltalkScriptsToExecute::new();

        if let Some((private, public)) = ssh_keys {
            scripts_to_execute.add(
                SmalltalkExpressionBuilder::new()
                    .add("IceCredentialsProvider useCustomSsh: true")
                    .add(format!(
                        "IceCredentialsProvider sshCredentials publicKey: '{}'; privateKey: '{}'",
                        private.display(),
                        public.display()
                    ))
                    .build(),
            );
        }

        scripts_to_execute
            .add(SmalltalkScriptToExecute::new("load-gt.st"))
            .execute(gtoolkit.evaluator().save(true))
            .await?;

        println!("{} Done in {}", SPARKLE, HumanDuration(started.elapsed()));

        Ok(())
    }
}
