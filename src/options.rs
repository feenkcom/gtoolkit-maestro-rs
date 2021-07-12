use clap::{AppSettings, Clap};
use std::path::PathBuf;

pub const DEFAULT_REPOSITORY: &str = "https://github.com/feenkcom/gtoolkit.git";
pub const DEFAULT_BRANCH: &str = "main";
pub const DEFAULT_DIRECTORY: &str = "glamoroustoolkit";

#[derive(Clap, Clone, Debug)]
#[clap(author = "feenk gmbh <contact@feenk.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct AppOptions {
    /// A name of the environment variable with personal GitHub token. The reason we do not accept tokens directly is because then it would be exposed in the CI log
    #[clap(long)]
    token: Option<String>,
    #[clap(subcommand)]
    sub_command: SubCommand,
}

#[derive(Clap, Clone, Debug)]
pub enum SubCommand {
    Build(BuildOptions),
    Get,
    Clone,
}

/// Builds GlamorousToolkit from sources
#[derive(Clap, Debug, Clone)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct BuildOptions {
    /// Delete existing installation of the gtoolkit if present
    #[clap(long)]
    overwrite: bool,
}

#[derive(Clone, Debug)]
#[repr(u32)]
pub enum PlatformOS {
    MacOSX8664,
    MacOSAarch64,
    WindowsX8664,
    LinuxX8664
}

impl AppOptions {
    pub fn command(&self) -> SubCommand {
        self.sub_command.clone()
    }

    pub fn repository(&self) -> String {
        DEFAULT_REPOSITORY.to_owned()
    }

    pub fn branch(&self) -> String {
        DEFAULT_BRANCH.to_owned()
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
            (os, arch) => { panic!("Unsupported {}-{}", os, arch); }
        }
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
        "https://files.pharo.org/get-files/90/pharo64.zip"
    }

    pub fn should_overwrite(&self) -> bool {
        match &self.sub_command {
            SubCommand::Build(build) => build.overwrite,
            SubCommand::Get => false,
            SubCommand::Clone => false,
        }
    }

    pub fn pharo_executable(&self) -> PathBuf {
        PathBuf::from(match self.platform() {
            PlatformOS::MacOSX8664 => { "pharo-vm/Pharo.app/Contents/MacOS/Pharo" }
            PlatformOS::MacOSAarch64 => { "pharo-vm/Pharo.app/Contents/MacOS/Pharo" }
            PlatformOS::WindowsX8664 => { "pharo-vm/PharoConsole.exe" }
            PlatformOS::LinuxX8664 => { "pharo-vm/pharo" }
        })
    }

    pub fn gtoolkit_app_cli(&self) -> PathBuf {
        PathBuf::from(match self.platform() {
            PlatformOS::MacOSX8664 => { "GlamorousToolkit.app/Contents/MacOS/GlamorousToolkit-cli" }
            PlatformOS::MacOSAarch64 => { "GlamorousToolkit.app/Contents/MacOS/GlamorousToolkit-cli" }
            PlatformOS::WindowsX8664 => { "GlamorousToolkit/bin/GlamorousToolkit-cli.exe" }
            PlatformOS::LinuxX8664 => { "GlamorousToolkit/bin/GlamorousToolkit-cli" }
        })
    }

    pub fn gtoolkit_image(&self) -> PathBuf {
        self.gtoolkit_directory().join("GlamorousToolkit.image")
    }
}
