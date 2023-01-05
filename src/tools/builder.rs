use crate::create::FileToCreate;
use crate::{
    Application, Checker, Downloader, ExecutableSmalltalk, FileToMove, ImageSeed, InstallerError,
    Result, Smalltalk, SmalltalkCommand, SmalltalkExpressionBuilder, SmalltalkFlags,
    SmalltalkScriptToExecute, SmalltalkScriptsToExecute, BUILDING, CREATING, DEFAULT_PHARO_IMAGE,
    DOWNLOADING, EXTRACTING, MOVING, SPARKLE,
};
use clap::{ArgEnum, Parser};
use downloader::{FileToDownload, FilesToDownload};
use feenk_releaser::{Version, VersionBump};
use file_matcher::FileNamed;
use indicatif::HumanDuration;
use reqwest::StatusCode;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;
use unzipper::{FileToUnzip, FilesToUnzip};
use url::Url;

#[derive(Parser, Debug, Clone)]
pub struct BuildOptions {
    /// Delete existing installation of the gtoolkit if present
    #[clap(long)]
    pub overwrite: bool,
    #[clap(long, default_value = "cloner", arg_enum, ignore_case = true)]
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
    /// Specify a URL to a pharo vm which will be used to prepare a seed image
    #[clap(long, parse(try_from_str = url_parse))]
    pub pharo_vm_url: Option<Url>,
    /// Public ssh key to use when cloning repositories
    #[clap(long, parse(from_os_str))]
    pub public_key: Option<PathBuf>,
    /// Private ssh key to use when cloning repositories
    #[clap(long, parse(from_os_str))]
    pub private_key: Option<PathBuf>,
    /// Specify a named version to load: 'bleeding-edge', 'latest-release' or 'vX.Y.Z'
    #[clap(long, parse(try_from_str = BuildVersion::from_str), default_value = BuildVersion::BleedingEdge.abstract_name())]
    pub version: BuildVersion,
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

#[derive(Parser, Debug, Clone)]
pub struct ReleaseBuildOptions {
    #[clap(flatten)]
    pub build_options: BuildOptions,
    /// When building an image for a release, specify which component version to bump
    #[clap(long, default_value = VersionBump::Patch.to_str(), possible_values = VersionBump::variants(), ignore_case = true)]
    pub bump: VersionBump,
    /// Do not open a default GtWorld
    #[clap(long)]
    pub no_gt_world: bool,
}

#[derive(Parser, Debug, Clone)]
pub struct LocalBuildOptions {
    #[clap(flatten)]
    pub build_options: BuildOptions,
    /// Do not open a default GtWorld
    #[clap(long)]
    pub no_gt_world: bool,
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
            pharo_vm_url: None,
            public_key: None,
            private_key: None,
            version: BuildVersion::BleedingEdge,
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
        self.to_possible_value().unwrap().get_name().to_string()
    }
}

#[derive(Debug, Clone)]
pub enum BuildVersion {
    LatestRelease,
    BleedingEdge,
    Version(Version),
}

impl BuildVersion {
    pub fn abstract_name(&self) -> &str {
        match self {
            BuildVersion::LatestRelease => "latest-release",
            BuildVersion::BleedingEdge => "bleeding-edge",
            BuildVersion::Version(_) => "vX.Y.Z",
        }
    }
}

impl FromStr for BuildVersion {
    type Err = InstallerError;

    fn from_str(s: &str) -> Result<Self> {
        let version = s.to_string().to_lowercase();
        let version_str = version.as_str();
        match version_str {
            "latest-release" => Ok(BuildVersion::LatestRelease),
            "bleeding-edge" => Ok(BuildVersion::BleedingEdge),
            _ => Ok(BuildVersion::Version(Version::parse(version_str)?)),
        }
    }
}

impl ToString for BuildVersion {
    fn to_string(&self) -> String {
        match self {
            BuildVersion::Version(version) => version.to_string(),
            _ => self.abstract_name().to_string(),
        }
    }
}

pub struct Builder;

#[derive(Serialize)]
pub struct LoaderVersionInfo {
    gtoolkit_version: String,
    releaser_version: String,
}

impl Builder {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn resolve_loader_version_info(
        &self,
        build_options: &BuildOptions,
    ) -> Result<LoaderVersionInfo> {
        let gtoolkit_version_string = match &build_options.version {
            BuildVersion::LatestRelease => {
                format!(
                    "v{}",
                    Application::latest_gtoolkit_image_version()
                        .await?
                        .to_string()
                )
            }
            BuildVersion::BleedingEdge => "main".to_string(),
            BuildVersion::Version(version) => {
                format!("v{}", version.to_string())
            }
        };

        let releaser_version_string = match &build_options.version {
            BuildVersion::BleedingEdge => "main".to_string(),
            _ => {
                let releaser_version_file_url_string = format!(
                    "https://raw.githubusercontent.com/feenkcom/gtoolkit/{}/gtoolkit-releaser.version",
                    &gtoolkit_version_string
                );

                let releaser_version_file_url = Url::parse(&releaser_version_file_url_string)?;

                let releaser_version_file_response =
                    reqwest::get(releaser_version_file_url.clone()).await?;
                if releaser_version_file_response.status() != StatusCode::OK {
                    return InstallerError::FailedToDownloadReleaserVersion(
                        releaser_version_file_url.clone(),
                        releaser_version_file_response.status(),
                    )
                    .into();
                }

                let releaser_version_file_content = releaser_version_file_response.text().await?;
                let releaser_version = Version::parse(releaser_version_file_content)?;
                format!("v{}", releaser_version.to_string())
            }
        };

        Ok(LoaderVersionInfo {
            gtoolkit_version: gtoolkit_version_string,
            releaser_version: releaser_version_string,
        })
    }

    fn pharo_vm_url(
        &self,
        application: &mut Application,
        build_options: &BuildOptions,
    ) -> Result<Url> {
        if let Some(ref custom_vm_url) = build_options.pharo_vm_url {
            Ok(custom_vm_url.clone())
        } else {
            Url::parse(application.pharo_vm_url()).map_err(|err| err.into())
        }
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
            self.pharo_vm_url(application, build_options)?,
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
            let seed_smalltalk = Smalltalk::new(
                application.pharo_executable(),
                seed_image,
                SmalltalkFlags::pharo(),
                application,
            );
            let mut seed_evaluator = seed_smalltalk.evaluator();
            seed_evaluator.interactive(false);

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

        let loader_template_string = match build_options.loader {
            Loader::Cloner => include_str!("../st/clone-gt.st"),
            Loader::Metacello => include_str!("../st/load-gt.st"),
        };

        let loader_template = mustache::compile_str(loader_template_string)?;
        let loader_version_info = self.resolve_loader_version_info(build_options).await?;
        let loader_script = loader_template.render_to_string(&loader_version_info)?;
        let loader_script_file_name =
            format!("load-gt-{}.st", &loader_version_info.gtoolkit_version);

        println!("{}Creating build scripts...", CREATING);
        FileToCreate::new(
            application.workspace().join("load-patches.st"),
            include_str!("../st/load-patches.st"),
        )
        .create()
        .await?;
        FileToCreate::new(
            application.workspace().join(&loader_script_file_name),
            loader_script,
        )
        .create()
        .await?;

        let gtoolkit = application.gtoolkit();
        let pharo = application.pharo();

        println!("{}Preparing the image...", BUILDING);
        SmalltalkScriptsToExecute::new()
            .add(SmalltalkScriptToExecute::new("load-patches.st"))
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
            .add(SmalltalkScriptToExecute::new(&loader_script_file_name))
            .execute(gtoolkit.evaluator().save(true))
            .await?;

        println!("{} Done in {}", SPARKLE, HumanDuration(started.elapsed()));

        Ok(())
    }
}
