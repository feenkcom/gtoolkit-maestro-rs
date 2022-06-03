use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::{
    AppVersion, ImageSeed, ImageVersion, InstallerError, Result, Smalltalk, SmalltalkFlags,
};
use feenk_releaser::{GitHub, Version};
use file_matcher::{FolderNamed, OneEntry, OneEntryNamed};
use std::fs::File;
use std::io::Write;

pub const DEFAULT_IMAGE_NAME: &str = "GlamorousToolkit";
pub const DEFAULT_IMAGE_EXTENSION: &str = "image";

pub const DEFAULT_PHARO_VM_MAC: &str = "https://dl.feenk.com/pharo/pharo64-mac-headless-stable.zip";
pub const DEFAULT_PHARO_VM_LINUX: &str =
    "https://dl.feenk.com/pharo/pharo64-linux-headless-stable.zip";
pub const DEFAULT_PHARO_VM_WINDOWS: &str =
    "https://dl.feenk.com/pharo/pharo64-win-headless-stable.zip";

pub const DEFAULT_PHARO_IMAGE: &str =
    "https://dl.feenk.com/pharo/Pharo9.0-SNAPSHOT.build.1575.sha.9bb5f99.arch.64bit.zip";

pub const SERIALIZATION_FILE: &str = "gtoolkit.yaml";

pub const GTOOLKIT_REPOSITORY_OWNER: &str = "feenkcom";
pub const GTOOLKIT_REPOSITORY_NAME: &str = "gtoolkit";

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
    pub fn new(
        workspace: impl AsRef<Path>,
        app_version: AppVersion,
        image_version: ImageVersion,
        image_seed: ImageSeed,
    ) -> Result<Self> {
        let workspace = workspace.as_ref();
        let workspace = if workspace.is_relative() {
            std::env::current_dir()?.join(workspace)
        } else {
            workspace.to_path_buf()
        };

        Ok(Self {
            verbose: false,
            workspace,
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

    pub fn serialization_file_name(&self) -> &str {
        SERIALIZATION_FILE
    }

    pub fn serialization_file(&self) -> PathBuf {
        self.workspace().join(self.serialization_file_name())
    }

    pub fn serialize_into_file(&self) -> Result<()> {
        let mut file = File::create(self.serialization_file())?;
        file.write(serde_yaml::to_string(self)?.as_bytes())?;
        Ok(())
    }

    pub fn deserialize_from_file(&mut self) -> Result<()> {
        let application: Self =
            serde_yaml::from_str(std::fs::read_to_string(self.serialization_file())?.as_str())
                .map_err(|error| Into::<InstallerError>::into(error))?;

        self.image_extension = application.image_extension;
        self.image_name = application.image_name;
        self.image_seed = application.image_seed;
        self.app_version = application.app_version;
        self.image_version = application.image_version;

        Ok(())
    }

    pub fn platform(&self) -> PlatformOS {
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;

        match (os, arch) {
            ("macos", "aarch64") => PlatformOS::MacOSAarch64,
            ("macos", "x86_64") => PlatformOS::MacOSX8664,
            ("linux", "x86_64") => PlatformOS::LinuxX8664,
            ("windows", "x86_64") => PlatformOS::WindowsX8664,
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
            PlatformOS::LinuxX8664 => {
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
            PlatformOS::MacOSX8664 => DEFAULT_PHARO_VM_MAC,
            PlatformOS::MacOSAarch64 => DEFAULT_PHARO_VM_MAC,
            PlatformOS::WindowsX8664 => DEFAULT_PHARO_VM_WINDOWS,
            PlatformOS::LinuxX8664 => DEFAULT_PHARO_VM_LINUX,
        }
    }

    pub fn gtoolkit_app(&self) -> &str {
        match self.platform() {
            PlatformOS::MacOSX8664 => "GlamorousToolkit.app/Contents/MacOS/GlamorousToolkit",
            PlatformOS::MacOSAarch64 => "GlamorousToolkit.app/Contents/MacOS/GlamorousToolkit",
            PlatformOS::WindowsX8664 => "bin/GlamorousToolkit.exe",
            PlatformOS::LinuxX8664 => "bin/GlamorousToolkit",
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
            PlatformOS::LinuxX8664 => {
                format!("https://github.com/feenkcom/gtoolkit-vm/releases/download/v{}/GlamorousToolkit-x86_64-unknown-linux-gnu.zip", &version)
            }
        }
    }

    pub fn gtoolkit_app_entries(&self) -> Vec<Box<dyn OneEntryNamed>> {
        match self.platform() {
            PlatformOS::MacOSX8664 | PlatformOS::MacOSAarch64 => {
                vec![FolderNamed::wildmatch("*.app").boxed()]
            }
            PlatformOS::WindowsX8664 => {
                vec![FolderNamed::exact("bin").boxed()]
            }
            PlatformOS::LinuxX8664 => {
                vec![
                    FolderNamed::exact("bin").boxed(),
                    FolderNamed::exact("lib").boxed(),
                ]
            }
        }
    }

    pub fn pharo_executable(&self) -> PathBuf {
        PathBuf::from(match self.platform() {
            PlatformOS::MacOSX8664 => "pharo-vm/Pharo.app/Contents/MacOS/Pharo",
            PlatformOS::MacOSAarch64 => "pharo-vm/Pharo.app/Contents/MacOS/Pharo",
            PlatformOS::WindowsX8664 => "pharo-vm/PharoConsole.exe",
            PlatformOS::LinuxX8664 => "pharo-vm/pharo",
        })
    }

    pub fn gtoolkit_app_cli(&self) -> PathBuf {
        PathBuf::from(match self.platform() {
            PlatformOS::MacOSX8664 => "GlamorousToolkit.app/Contents/MacOS/GlamorousToolkit-cli",
            PlatformOS::MacOSAarch64 => "GlamorousToolkit.app/Contents/MacOS/GlamorousToolkit-cli",
            PlatformOS::WindowsX8664 => "bin/GlamorousToolkit-cli.exe",
            PlatformOS::LinuxX8664 => "bin/GlamorousToolkit-cli",
        })
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
    LinuxX8664,
}
