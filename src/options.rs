use crate::{BuildOptions, SetupOptions, Smalltalk, TentativeOptions, TestOptions};
use clap::{AppSettings, Clap};
use feenk_releaser::{GitHub, Version};
use std::error::Error;
use std::path::PathBuf;

pub const DEFAULT_REPOSITORY: &str = "https://github.com/feenkcom/gtoolkit.git";
pub const DEFAULT_BRANCH: &str = "main";
pub const DEFAULT_DIRECTORY: &str = "glamoroustoolkit";

pub const VM_REPOSITORY_OWNER: &str = "feenkcom";
pub const VM_REPOSITORY_NAME: &str = "gtoolkit-vm";

pub const DEFAULT_PHARO_IMAGE: &str =
    "https://files.pharo.org/image/90/Pharo9.0-SNAPSHOT.build.1532.sha.e58ef49.arch.64bit.zip";

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
    /// Specify the version of the VM. When not specified, will use the latest released version
    #[clap(long, parse(try_from_str = version_parse))]
    vm_version: Option<Version>,
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
    ReleaseBuild,
    /// Builds GlamorousToolkit image from sources without performing any extra setup.
    #[clap(display_order = 3)]
    Build(BuildOptions),
    /// Sets up the GlamorousToolkit image. This includes opening a default GtWorld and configuring various settings.
    #[clap(display_order = 4)]
    Setup(SetupOptions),
    /// Tests Glamorous Toolkit and exports the results.
    #[clap(display_order = 5)]
    Test(TestOptions),
    /// Package the GlamorousToolkit image as a tentative release.
    #[clap(display_order = 6)]
    PackageTentative(TentativeOptions),
    /// Given a packaged tentative image, download the GlamorousToolkit app for the version specified in the .version file
    #[clap(display_order = 7)]
    UnpackageTentative(TentativeOptions),
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

    pub fn vm_version(&self) -> Option<&Version> {
        self.vm_version.as_ref()
    }

    pub fn vm_version_file_name(&self) -> &str {
        "gtoolkit-vm.version"
    }

    pub fn vm_version_file(&self) -> PathBuf {
        self.gtoolkit_directory().join(self.vm_version_file_name())
    }

    pub fn repository(&self) -> String {
        DEFAULT_REPOSITORY.to_owned()
    }

    pub fn branch(&self) -> String {
        DEFAULT_BRANCH.to_owned()
    }

    pub fn gtoolkit(&self) -> Smalltalk {
        Smalltalk::new(self.gtoolkit_app_cli(), self.gtoolkit_image())
            .set_workspace(self.gtoolkit_directory())
            .set_options(self.clone())
    }

    pub fn pharo(&self) -> Smalltalk {
        Smalltalk::new(self.pharo_executable(), self.gtoolkit_image())
            .set_workspace(self.gtoolkit_directory())
            .set_options(self.clone())
    }

    pub fn gtoolkit_directory(&self) -> PathBuf {
        std::env::current_dir().unwrap().join(DEFAULT_DIRECTORY)
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
            PlatformOS::MacOSX8664 => {
                "https://files.pharo.org/get-files/90/pharo64-mac-headless-stable.zip"
            }
            PlatformOS::MacOSAarch64 => {
                "https://files.pharo.org/get-files/90/pharo64-mac-headless-stable.zip"
            }
            PlatformOS::WindowsX8664 => {
                "https://files.pharo.org/get-files/90/pharo64-win-headless-latest.zip"
            }
            PlatformOS::LinuxX8664 => {
                "https://files.pharo.org/get-files/90/pharo64-linux-headless-latest.zip"
            }
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
            PlatformOS::WindowsX8664 => "bin\\GlamorousToolkit.exe",
            PlatformOS::LinuxX8664 => "./bin/GlamorousToolkit",
        }
    }

    pub fn gtoolkit_image(&self) -> PathBuf {
        self.gtoolkit_directory().join("GlamorousToolkit.image")
    }
}
