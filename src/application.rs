use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use feenk_releaser::{GitHub, Version};
use file_matcher::{FolderNamed, OneEntry, OneEntryNamed};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::options::{VM_REPOSITORY_NAME, VM_REPOSITORY_OWNER};
use crate::{
    AppVersion, ImageSeed, ImageVersion, InstallerError, Result, Smalltalk, SmalltalkFlags,
    DEFAULT_IMAGE_EXTENSION, DEFAULT_IMAGE_NAME, DEFAULT_PHARO_IMAGE,
    DEFAULT_PHARO_VM_LINUX_AARCH64, DEFAULT_PHARO_VM_LINUX_X86_64, DEFAULT_PHARO_VM_MAC_AARCH64,
    DEFAULT_PHARO_VM_MAC_X86_64, DEFAULT_PHARO_VM_WINDOWS, GTOOLKIT_REPOSITORY_NAME,
    GTOOLKIT_REPOSITORY_OWNER, SERIALIZATION_FILE,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Application {
    verbose: bool,
    workspace: PathBuf,
    app_version: AppVersion,
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

    pub fn pharo(&self) -> Smalltalk {
        Smalltalk::new(
            self.pharo_executable(),
            self.image(),
            SmalltalkFlags::pharo(),
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

    pub fn platform(&self) -> PlatformOS {
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

    pub fn gtoolkit_app_folders(&self) -> Vec<OneEntry> {
        let folders = match self.platform() {
            PlatformOS::MacOSX8664 => {
                vec![FolderNamed::exact("GlamorousToolkit.app")]
            }
            PlatformOS::MacOSAarch64 => {
                vec![FolderNamed::exact("GlamorousToolkit.app")]
            }
            PlatformOS::WindowsX8664 => {
                vec![FolderNamed::exact("bin")]
            }
            PlatformOS::WindowsAarch64 => {
                vec![FolderNamed::exact("bin")]
            }
            PlatformOS::LinuxX8664 => {
                vec![FolderNamed::exact("bin"), FolderNamed::exact("lib")]
            }
            PlatformOS::LinuxAarch64 => {
                vec![FolderNamed::exact("bin"), FolderNamed::exact("lib")]
            }
        };

        folders
            .into_iter()
            .map(|each| each.within(self.workspace()))
            .collect::<Vec<OneEntry>>()
    }

    pub fn pharo_vm_url(&self) -> &str {
        match self.platform() {
            PlatformOS::MacOSX8664 => DEFAULT_PHARO_VM_MAC_X86_64,
            PlatformOS::MacOSAarch64 => DEFAULT_PHARO_VM_MAC_AARCH64,
            PlatformOS::WindowsX8664 => DEFAULT_PHARO_VM_WINDOWS,
            PlatformOS::WindowsAarch64 => DEFAULT_PHARO_VM_WINDOWS,
            PlatformOS::LinuxX8664 => DEFAULT_PHARO_VM_LINUX_X86_64,
            PlatformOS::LinuxAarch64 => DEFAULT_PHARO_VM_LINUX_AARCH64,
        }
    }

    pub fn gtoolkit_app(&self) -> &str {
        match self.platform() {
            PlatformOS::MacOSX8664 | PlatformOS::MacOSAarch64 => {
                "GlamorousToolkit.app/Contents/MacOS/GlamorousToolkit"
            }
            PlatformOS::WindowsX8664 | PlatformOS::WindowsAarch64 => "bin/GlamorousToolkit.exe",
            PlatformOS::LinuxX8664 | PlatformOS::LinuxAarch64 => "bin/GlamorousToolkit",
        }
    }

    pub fn gtoolkit_app_url(&self) -> String {
        let version = self.app_version().to_string();
        match self.platform() {
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
        }
    }

    pub fn gtoolkit_app_entries(&self) -> Vec<Box<dyn OneEntryNamed>> {
        match self.platform() {
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
        }
    }

    pub fn pharo_executable(&self) -> PathBuf {
        PathBuf::from(match self.platform() {
            PlatformOS::MacOSX8664 | PlatformOS::MacOSAarch64 => {
                "pharo-vm/Pharo.app/Contents/MacOS/Pharo"
            }
            PlatformOS::WindowsX8664 | PlatformOS::WindowsAarch64 => "pharo-vm/PharoConsole.exe",
            PlatformOS::LinuxX8664 | PlatformOS::LinuxAarch64 => "pharo-vm/pharo",
        })
    }

    pub fn gtoolkit_app_cli(&self) -> PathBuf {
        PathBuf::from(match self.platform() {
            PlatformOS::MacOSX8664 | PlatformOS::MacOSAarch64 => {
                "GlamorousToolkit.app/Contents/MacOS/GlamorousToolkit-cli"
            }
            PlatformOS::WindowsX8664 | PlatformOS::WindowsAarch64 => "bin/GlamorousToolkit-cli.exe",
            PlatformOS::LinuxX8664 | PlatformOS::LinuxAarch64 => "bin/GlamorousToolkit-cli",
        })
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

#[derive(Clone, Debug)]
#[repr(u32)]
pub enum PlatformOS {
    MacOSX8664,
    MacOSAarch64,
    WindowsX8664,
    WindowsAarch64,
    LinuxX8664,
    LinuxAarch64,
}
