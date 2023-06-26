use crate::{Application, PlatformOS};
use file_matcher::{FolderNamed, OneEntry};

pub struct Package;

impl Package {
    pub fn gtoolkit_app_folders(application: &Application) -> Vec<OneEntry> {
        Self::gtoolkit_app_entries_for_target(application, application.host_platform())
    }

    pub fn gtoolkit_app_entries_for_target(
        application: &Application,
        target: PlatformOS,
    ) -> Vec<OneEntry> {
        let folders = match target {
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
            PlatformOS::AndroidAarch64 => {
                vec![FolderNamed::exact("lib")]
            }
        };

        folders
            .into_iter()
            .map(|each| each.within(application.gtoolkit_app_location(target)))
            .collect::<Vec<OneEntry>>()
    }
}
