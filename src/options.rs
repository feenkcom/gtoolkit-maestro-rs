use crate::{
    BuildOptions, CopyOptions, ReleaseBuildOptions, ReleaseOptions, SetupOptions, Smalltalk,
    StartOptions, TentativeOptions, TestOptions,
};
use clap::{AppSettings, Clap};
use feenk_releaser::{GitHub, Version, VersionBump};
use file_matcher::{FolderNamed, OneEntry};
use std::error::Error;
use std::path::PathBuf;

pub const DEFAULT_REPOSITORY: &str = "https://github.com/feenkcom/gtoolkit.git";
pub const DEFAULT_BRANCH: &str = "main";
pub const DEFAULT_DIRECTORY: &str = "glamoroustoolkit";

pub const GTOOLKIT_REPOSITORY_OWNER: &str = "feenkcom";
pub const GTOOLKIT_REPOSITORY_NAME: &str = "gtoolkit";

pub const VM_REPOSITORY_OWNER: &str = "feenkcom";
pub const VM_REPOSITORY_NAME: &str = "gtoolkit-vm";

pub const DEFAULT_PHARO_IMAGE: &str =
    "https://dl.feenk.com/pharo/Pharo9.0-SNAPSHOT.build.1532.sha.e58ef49.arch.64bit.zip";
pub const DEFAULT_PHARO_VM_MAC: &str = "https://dl.feenk.com/pharo/pharo64-mac-headless-stable.zip";
pub const DEFAULT_PHARO_VM_LINUX: &str =
    "https://dl.feenk.com/pharo/pharo64-linux-headless-stable.zip";
pub const DEFAULT_PHARO_VM_WINDOWS: &str =
    "https://dl.feenk.com/pharo/pharo64-win-headless-stable.zip";

#[derive(Clap, Clone, Debug)]
#[clap(author = "feenk gmbh <contact@feenk.com>")]
#[clap(setting = AppSettings::ColorAlways)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct AppOptions {
    #[clap(subcommand)]
    sub_command: SubCommand,
    /// Perform commands in a verbose manner
    #[clap(long)]
    verbose: bool,
    #[clap(long, default_value = DEFAULT_DIRECTORY, parse(from_os_str))]
    workspace: PathBuf,
    /// Specify the version of the VM. When not specified, will use the latest released version
    #[clap(long, parse(try_from_str = version_parse))]
    vm_version: Option<Version>,
    gtoolkit_version: Option<Version>,
    /// Specify a URl from which to download a clean pharo image. When not specified, will use some hardcoded value
    #[clap(long)]
    image_url: Option<String>,
}

#[derive(Clap, Clone, Debug)]
pub enum SubCommand {
    /// Creates a typical local build of GlamorousToolkit with GtWorld opened and sets the image up. This is intended to be used by developers and contributors.
    #[clap(display_order = 1)]
    LocalBuild,
    /// Creates a release build of GlamorousToolkit with GtWorld opened and sets up the image to be deployed. This is intended to be used by the Continuous Integration server.
    #[clap(display_order = 2)]
    ReleaseBuild(ReleaseBuildOptions),
    /// Builds GlamorousToolkit image from sources without performing any extra setup.
    #[clap(display_order = 3)]
    Build(BuildOptions),
    /// Sets up the GlamorousToolkit image. This includes opening a default GtWorld and configuring various settings.
    #[clap(display_order = 4)]
    Setup(SetupOptions),
    /// Copies glamorous toolkit related files from the current workspace into a new workspace.
    /// Does not copy temporary files or logs
    #[clap(display_order = 5)]
    CopyTo(CopyOptions),
    /// Starts an application interactively, waits for a duration of delay to let it load completely then saves and quits.
    #[clap(display_order = 6)]
    Start(StartOptions),
    /// Cleans up an image after loading Glamorous Toolkit. It cleans up ssh keys, removes iceberg repositories
    /// and garbage collects objects
    #[clap(display_order = 7)]
    CleanUp,
    /// Tests Glamorous Toolkit and exports the results.
    #[clap(display_order = 8)]
    Test(TestOptions),
    /// Package the GlamorousToolkit image as a tentative release.
    #[clap(display_order = 9)]
    PackageTentative(TentativeOptions),
    /// Given a packaged tentative image, download the GlamorousToolkit app for the version specified in the .version file
    #[clap(display_order = 10)]
    UnpackageTentative(TentativeOptions),
    /// Package the GlamorousToolkit image and App for a release. Prints the path to the created package in the `stdout`
    #[clap(display_order = 11)]
    PackageRelease(ReleaseOptions),
    /// Run the gtoolkit-releaser to release glamorous toolkit
    #[clap(display_order = 12)]
    RunReleaser,
    /// Display the Debug information of the AppOptions
    #[clap(display_order = 13)]
    PrintDebug,
}

#[derive(Clone, Debug)]
#[repr(u32)]
pub enum PlatformOS {
    MacOSX8664,
    MacOSAarch64,
    WindowsX8664,
    LinuxX8664,
}

fn version_parse(val: &str) -> Result<Version, Box<dyn Error>> {
    Version::parse(val)
}

impl AppOptions {
    pub fn command(&self) -> SubCommand {
        self.sub_command.clone()
    }

    pub async fn ensure_vm_version(&mut self) -> Result<(), Box<dyn Error>> {
        if self.vm_version.is_some() {
            return Ok(());
        }

        if self.vm_version_file().exists() {
            self.read_vm_version().await?;
            return Ok(());
        }

        let latest_version: Option<Version> =
            GitHub::new(VM_REPOSITORY_OWNER, VM_REPOSITORY_NAME, None)
                .latest_release_version()
                .await?;
        if let Some(latest_version) = latest_version {
            self.vm_version = Some(latest_version);
            Ok(())
        } else {
            Err(Box::new(crate::error::Error {
                what: "VM is not yet released".to_string(),
                source: None,
            }))
        }
    }

    pub async fn ensure_gtoolkit_version(&mut self) -> Result<(), Box<dyn Error>> {
        if self.gtoolkit_version.is_some() {
            return Ok(());
        }

        if self.gtoolkit_version_file().exists() {
            self.read_gtoolkit_version().await?;
            return Ok(());
        }

        let latest_version: Option<Version> =
            GitHub::new(GTOOLKIT_REPOSITORY_OWNER, GTOOLKIT_REPOSITORY_NAME, None)
                .latest_release_version()
                .await?;
        let gtoolkit_version = if let Some(latest_version) = latest_version {
            latest_version
        } else {
            Version::new(VersionBump::Patch)
        };
        self.gtoolkit_version = Some(gtoolkit_version);
        Ok(())
    }

    /// Read the vm version from the vm version file
    pub async fn read_vm_version(&mut self) -> Result<(), Box<dyn Error>> {
        let vm_version_file_path = self.vm_version_file();

        if !vm_version_file_path.exists() {
            return Err(Box::new(crate::error::Error {
                what: format!("Cound not find {:?}", &vm_version_file_path),
                source: None,
            }));
        }

        let version_string = std::fs::read_to_string(&vm_version_file_path)?;
        let version = Version::parse(version_string)?;

        self.vm_version = Some(version);

        Ok(())
    }

    /// Read the gtoolkit version from the vm version file
    pub async fn read_gtoolkit_version(&mut self) -> Result<(), Box<dyn Error>> {
        let gtoolkit_version_file_path = self.gtoolkit_version_file();

        if !gtoolkit_version_file_path.exists() {
            return Err(Box::new(crate::error::Error {
                what: format!("Cound not find {:?}", &gtoolkit_version_file_path),
                source: None,
            }));
        }

        let version_string = std::fs::read_to_string(&gtoolkit_version_file_path)?;
        let version = Version::parse(version_string)?;

        self.gtoolkit_version = Some(version);

        Ok(())
    }

    pub fn vm_version(&self) -> Option<&Version> {
        self.vm_version.as_ref()
    }

    pub fn gtoolkit_version(&self) -> Option<&Version> {
        self.gtoolkit_version.as_ref()
    }

    pub fn vm_version_file_name(&self) -> &str {
        "gtoolkit-vm.version"
    }

    pub fn gtoolkit_version_file_name(&self) -> &str {
        "gtoolkit.version"
    }

    pub fn vm_version_file(&self) -> PathBuf {
        self.workspace().join(self.vm_version_file_name())
    }

    pub fn gtoolkit_version_file(&self) -> PathBuf {
        self.workspace().join(self.gtoolkit_version_file_name())
    }

    pub fn repository(&self) -> String {
        DEFAULT_REPOSITORY.to_owned()
    }

    pub fn branch(&self) -> String {
        DEFAULT_BRANCH.to_owned()
    }

    pub fn gtoolkit(&self) -> Smalltalk {
        Smalltalk::new(self.gtoolkit_app_cli(), self.gtoolkit_image())
            .set_workspace(self.workspace())
            .set_options(self.clone())
    }

    pub fn pharo(&self) -> Smalltalk {
        Smalltalk::new(self.pharo_executable(), self.gtoolkit_image())
            .set_workspace(self.workspace())
            .set_options(self.clone())
    }

    pub fn workspace(&self) -> PathBuf {
        std::env::current_dir()
            .unwrap()
            .join(self.workspace.as_path())
    }

    pub fn set_workspace(&mut self, path: impl Into<PathBuf>) {
        self.workspace = path.into()
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

    pub fn verbose(&self) -> bool {
        self.verbose
    }

    pub fn pharo_vm_url(&self) -> &str {
        match self.platform() {
            PlatformOS::MacOSX8664 => DEFAULT_PHARO_VM_MAC,
            PlatformOS::MacOSAarch64 => DEFAULT_PHARO_VM_MAC,
            PlatformOS::WindowsX8664 => DEFAULT_PHARO_VM_WINDOWS,
            PlatformOS::LinuxX8664 => DEFAULT_PHARO_VM_LINUX,
        }
    }

    pub fn pharo_image_url(&self) -> &str {
        self.image_url
            .as_ref()
            .map(|url| url.as_ref())
            .unwrap_or(DEFAULT_PHARO_IMAGE)
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

    pub fn gtoolkit_app_version_string(&self) -> String {
        self.vm_version
            .as_ref()
            .expect("Version is not resolved")
            .to_string()
    }

    pub fn gtoolkit_app_url(&self) -> String {
        let version = self.gtoolkit_app_version_string();
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

    pub fn gtoolkit_app(&self) -> &str {
        match self.platform() {
            PlatformOS::MacOSX8664 => "GlamorousToolkit.app/Contents/MacOS/GlamorousToolkit",
            PlatformOS::MacOSAarch64 => "GlamorousToolkit.app/Contents/MacOS/GlamorousToolkit",
            PlatformOS::WindowsX8664 => "bin/GlamorousToolkit.exe",
            PlatformOS::LinuxX8664 => "bin/GlamorousToolkit",
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

    pub fn gtoolkit_image(&self) -> PathBuf {
        self.workspace().join("GlamorousToolkit.image")
    }
}
