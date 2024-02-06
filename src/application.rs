use clap::ArgEnum;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use feenk_releaser::{GitHub, Version};
use file_matcher::{FolderNamed, OneEntryNamed};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::options::{VM_REPOSITORY_NAME, VM_REPOSITORY_OWNER};
use crate::{
    AppVersion, GToolkit, ImageSeed, ImageVersion, InstallerError, Result, Smalltalk,
    SmalltalkFlags, DEFAULT_IMAGE_EXTENSION, DEFAULT_IMAGE_NAME, DEFAULT_PHARO_IMAGE, DOCKERFILE,
    DOCKER_IMAGE_CONTENT_DIRECTORY, GTOOLKIT_REPOSITORY_NAME, GTOOLKIT_REPOSITORY_OWNER,
    SERIALIZATION_FILE,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Application {
    verbose: bool,
    workspace: PathBuf,
    app_version: AppVersion,
    app_cli_binary: Option<PathBuf>,
    image_version: ImageVersion,
    image_name: String,
    image_extension: String,
    image_seed: ImageSeed,
}

impl Application {
    /// Try to setup an installer for a given workspace directory.
    /// First it probes it for the serialized state file and if it exists,
    /// the installer will be deserialized from it, otherwise it fetches
    /// the versions from the internet.
    pub async fn for_workspace(workspace: impl AsRef<Path>) -> Result<Self> {
        let workspace = workspace.as_ref();
        let serialization_file = workspace.join(Self::serialization_file_name());
        if serialization_file.exists() {
            Self::try_from_file(workspace, serialization_file.as_path())
        } else {
            Self::try_fetch_latest(workspace).await
        }
    }

    /// Deserializes an installer from the state file in the given workspace.
    /// Fails if the file does not exist
    pub fn for_workspace_from_file(workspace: impl AsRef<Path>) -> Result<Self> {
        let workspace = workspace.as_ref();
        let serialization_file = workspace.join(Self::serialization_file_name());
        Self::try_from_file(workspace, serialization_file.as_path())
    }

    fn try_from_file(
        workspace: impl AsRef<Path>,
        serialization_file: impl AsRef<Path>,
    ) -> Result<Self> {
        let serialization_file = serialization_file.as_ref();

        let file_content = std::fs::read_to_string(serialization_file).map_err(|error| {
            InstallerError::SerializationFileReadError(serialization_file.to_path_buf(), error)
        })?;

        let mut application: Application = serde_yaml::from_str(file_content.as_str())
            .map_err(|error| Into::<InstallerError>::into(error))?;

        application.workspace = workspace.as_ref().to_path_buf();
        Ok(application)
    }

    async fn try_fetch_latest(workspace: impl AsRef<Path>) -> Result<Self> {
        let gtoolkit_vm_version = Application::fetch_vm_version().await?;
        let gtoolkit_image_version = Application::latest_gtoolkit_image_version().await?;
        let image_seed = ImageSeed::Url(Url::parse(DEFAULT_PHARO_IMAGE)?);

        Self::new(
            workspace,
            gtoolkit_vm_version,
            gtoolkit_image_version,
            image_seed,
        )
    }

    fn new(
        workspace: impl AsRef<Path>,
        app_version: AppVersion,
        image_version: ImageVersion,
        image_seed: ImageSeed,
    ) -> Result<Self> {
        Ok(Self {
            verbose: false,
            workspace: workspace.as_ref().to_path_buf(),
            app_version,
            app_cli_binary: None,
            image_version,
            image_name: DEFAULT_IMAGE_NAME.to_string(),
            image_extension: DEFAULT_IMAGE_EXTENSION.to_string(),
            image_seed,
        })
    }

    pub fn is_verbose(&self) -> bool {
        self.verbose
    }

    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }

    pub fn workspace(&self) -> &Path {
        self.workspace.as_path()
    }

    pub fn set_workspace(&mut self, workspace: impl Into<PathBuf>) {
        self.workspace = workspace.into()
    }

    pub fn set_app_cli_binary(&mut self, binary: impl Into<PathBuf>) -> Result<()> {
        let binary = binary.into();
        self.app_cli_binary = Some(binary.clone());
        self.app_version = self.gtoolkit().get_app_version()?.into();
        Ok(())
    }

    pub fn has_explicit_app_cli_binary(&self) -> bool {
        self.app_cli_binary.is_some()
    }

    /// Returns a name of the image (without .image extension)
    pub fn image_name(&self) -> &str {
        self.image_name.as_str()
    }

    pub fn image_extension(&self) -> &str {
        self.image_extension.as_str()
    }

    pub fn image_seed(&self) -> &ImageSeed {
        &self.image_seed
    }

    pub fn set_image_seed(&mut self, seed: ImageSeed) -> Result<()> {
        match &seed {
            ImageSeed::Image(image_file) => {
                let seed_image_directory = seed.seed_image_directory(self);

                let workspace =
                    to_absolute::canonicalize(&seed_image_directory).map_err(|error| {
                        InstallerError::CanonicalizeError(seed_image_directory, error)
                    })?;

                self.set_workspace(workspace);

                let file_name = image_file
                    .file_stem()
                    .and_then(|name| name.to_str())
                    .and_then(|name| Some(name.to_string()));

                let file_extension = image_file
                    .extension()
                    .and_then(|name| name.to_str())
                    .and_then(|name| Some(name.to_string()));

                self.image_name = file_name
                    .ok_or_else(|| InstallerError::FailedToReadFileName(image_file.clone()))?;
                self.image_extension = file_extension
                    .ok_or_else(|| InstallerError::FailedToReadFileExtension(image_file.clone()))?;
            }
            _ => {}
        }

        self.image_seed = seed;
        Ok(())
    }

    /// Returns a path to the image with a glamorous application
    pub fn image(&self) -> PathBuf {
        self.workspace()
            .join(format!("{}.{}", self.image_name(), self.image_extension()))
    }

    pub fn image_version(&self) -> &ImageVersion {
        &self.image_version
    }

    pub fn set_image_version(&mut self, version: ImageVersion) {
        self.image_version = version;
    }

    pub fn set_app_version(&mut self, version: AppVersion) {
        self.app_version = version;
    }

    pub fn app_version(&self) -> &AppVersion {
        &self.app_version
    }

    pub fn gtoolkit(&self) -> Smalltalk {
        Smalltalk::new(
            self.gtoolkit_app_cli(),
            self.image(),
            SmalltalkFlags::gtoolkit(),
            self,
        )
    }

    pub fn serialization_file_name() -> &'static str {
        SERIALIZATION_FILE
    }

    pub fn serialization_file(&self) -> PathBuf {
        self.workspace().join(Self::serialization_file_name())
    }

    pub fn serialize_into_file(&self) -> Result<()> {
        let mut file = File::create(self.serialization_file())?;
        file.write(serde_yaml::to_string(self)?.as_bytes())?;
        Ok(())
    }

    pub fn dockerfile() -> &'static str {
        DOCKERFILE
    }

    pub fn docker_image_content_directory() -> &'static str {
        DOCKER_IMAGE_CONTENT_DIRECTORY
    }

    pub fn host_platform(&self) -> PlatformOS {
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;

        match (os, arch) {
            ("macos", "aarch64") => PlatformOS::MacOSAarch64,
            ("macos", "x86_64") => PlatformOS::MacOSX8664,
            ("linux", "x86_64") => PlatformOS::LinuxX8664,
            ("linux", "aarch64") => PlatformOS::LinuxAarch64,
            ("windows", "x86_64") => PlatformOS::WindowsX8664,
            ("windows", "aarch64") => PlatformOS::WindowsAarch64,
            (os, arch) => {
                panic!("Unsupported {}-{}", os, arch);
            }
        }
    }

    /// Return a location of the gtoolkit app (vm) for a given platform.
    /// If the target platform is the same as the host, the VM is placed in the workspace
    pub fn gtoolkit_app_location(&self, target: PlatformOS) -> PathBuf {
        if self.host_platform() == target {
            self.workspace().to_path_buf()
        } else {
            self.workspace().join(target.as_str())
        }
    }

    pub fn gtoolkit_app(&self) -> &str {
        match self.host_platform() {
            PlatformOS::MacOSX8664 | PlatformOS::MacOSAarch64 => {
                "GlamorousToolkit.app/Contents/MacOS/GlamorousToolkit"
            }
            PlatformOS::WindowsX8664 | PlatformOS::WindowsAarch64 => "bin/GlamorousToolkit.exe",
            PlatformOS::LinuxX8664 | PlatformOS::LinuxAarch64 => "bin/GlamorousToolkit",
            PlatformOS::AndroidAarch64 => {
                panic!("Installer is unable to run GlamorousToolkit on Android")
            }
        }
    }

    /// Return a URL of the gtoolkit app (VM) for the current host
    pub fn gtoolkit_app_host_url(&self) -> String {
        self.gtoolkit_app_url_for_target(self.host_platform())
    }

    pub fn gtoolkit_app_url_for_target(&self, platform: PlatformOS) -> String {
        let version = self.app_version().to_string();
        match platform {
            PlatformOS::MacOSX8664 => {
                format!("https://github.com/feenkcom/gtoolkit-vm/releases/download/v{}/GlamorousToolkit-x86_64-apple-darwin.app.zip", &version)
            }
            PlatformOS::MacOSAarch64 => {
                format!("https://github.com/feenkcom/gtoolkit-vm/releases/download/v{}/GlamorousToolkit-aarch64-apple-darwin.app.zip", &version)
            }
            PlatformOS::WindowsX8664 => {
                format!("https://github.com/feenkcom/gtoolkit-vm/releases/download/v{}/GlamorousToolkit-x86_64-pc-windows-msvc.zip", &version)
            }
            PlatformOS::WindowsAarch64 => {
                format!("https://github.com/feenkcom/gtoolkit-vm/releases/download/v{}/GlamorousToolkit-aarch64-pc-windows-msvc.zip", &version)
            }
            PlatformOS::LinuxX8664 => {
                format!("https://github.com/feenkcom/gtoolkit-vm/releases/download/v{}/GlamorousToolkit-x86_64-unknown-linux-gnu.zip", &version)
            }
            PlatformOS::LinuxAarch64 => {
                format!("https://github.com/feenkcom/gtoolkit-vm/releases/download/v{}/GlamorousToolkit-aarch64-unknown-linux-gnu.zip", &version)
            }
            PlatformOS::AndroidAarch64 => {
                format!("https://github.com/feenkcom/gtoolkit-vm/releases/download/v{}/GlamorousToolkit-aarch64-linux-android.apk", &version)
            }
        }
    }

    pub fn gtoolkit_app_entries(&self) -> Vec<Box<dyn OneEntryNamed>> {
        match self.host_platform() {
            PlatformOS::MacOSX8664 | PlatformOS::MacOSAarch64 => {
                vec![FolderNamed::wildmatch("*.app").boxed()]
            }
            PlatformOS::WindowsX8664 | PlatformOS::WindowsAarch64 => {
                vec![FolderNamed::exact("bin").boxed()]
            }
            PlatformOS::LinuxX8664 | PlatformOS::LinuxAarch64 => {
                vec![
                    FolderNamed::exact("bin").boxed(),
                    FolderNamed::exact("lib").boxed(),
                ]
            }
            PlatformOS::AndroidAarch64 => {
                vec![FolderNamed::exact("lib").boxed()]
            }
        }
    }

    pub fn gtoolkit_app_cli(&self) -> PathBuf {
        self.gtoolkit_app_cli_for_target(self.host_platform())
    }

    /// Return a path to the gtoolkit app's cli executable for a given platform.
    /// `app_cli_binary` overrides the path to the binary
    pub fn gtoolkit_app_cli_for_target(&self, target: PlatformOS) -> PathBuf {
        if let Some(ref cli) = self.app_cli_binary {
            return cli.clone();
        }

        let location = self.gtoolkit_app_location(target);
        let cli = PathBuf::from(match target {
            PlatformOS::MacOSX8664 | PlatformOS::MacOSAarch64 => {
                "GlamorousToolkit.app/Contents/MacOS/GlamorousToolkit-cli"
            }
            PlatformOS::WindowsX8664 | PlatformOS::WindowsAarch64 => "bin/GlamorousToolkit-cli.exe",
            PlatformOS::LinuxX8664 | PlatformOS::LinuxAarch64 => "bin/GlamorousToolkit-cli",
            PlatformOS::AndroidAarch64 => "lib/arm64-v8a/libvm_client_android.so",
        });
        location.join(cli)
    }

    pub async fn fetch_vm_version() -> Result<AppVersion> {
        let latest_version: Option<Version> =
            GitHub::new(VM_REPOSITORY_OWNER, VM_REPOSITORY_NAME, None)
                .latest_release_version()
                .await?;

        if let Some(latest_version) = latest_version {
            return Ok(latest_version.into());
        };

        InstallerError::FailedToDetectGlamorousAppVersion.into()
    }

    pub async fn latest_gtoolkit_image_version() -> Result<ImageVersion> {
        let latest_version: Option<Version> =
            GitHub::new(GTOOLKIT_REPOSITORY_OWNER, GTOOLKIT_REPOSITORY_NAME, None)
                .latest_release_version()
                .await?;

        if let Some(latest_version) = latest_version {
            return Ok(latest_version.into());
        };

        InstallerError::FailedToDetectGlamorousImageVersion.into()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
#[repr(u32)]
pub enum PlatformOS {
    #[clap(name = "x86_64-apple-darwin")]
    MacOSX8664,
    #[clap(name = "aarch64-apple-darwin")]
    MacOSAarch64,
    #[clap(name = "x86_64-pc-windows-msvc")]
    WindowsX8664,
    #[clap(name = "aarch64-pc-windows-msvc")]
    WindowsAarch64,
    #[clap(name = "x86_64-unknown-linux-gnu")]
    LinuxX8664,
    #[clap(name = "aarch64-unknown-linux-gnu")]
    LinuxAarch64,
    #[clap(name = "aarch64-linux-android")]
    AndroidAarch64,
}

impl PlatformOS {
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }

    pub fn is_android(&self) -> bool {
        match self {
            PlatformOS::AndroidAarch64 => true,
            _ => false,
        }
    }
}

impl AsRef<str> for PlatformOS {
    fn as_ref(&self) -> &str {
        self.to_possible_value().unwrap().get_name()
    }
}
