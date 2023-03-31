use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use clap::Parser;
use feenk_releaser::VersionBump;
use file_matcher::FileNamed;
use zipper::ToZip;

use crate::{Application, Downloader, ExecutableSmalltalk, PlatformOS, Result, SmalltalkCommand};

#[derive(Parser, Debug, Clone)]
pub struct ReleaseOptions {
    /// Path to the .zip with the release image build. Supports mustache syntax to inject various release related information.
    /// For example: "GlamorousToolkit-{{os}}-{{arch}}-v{{version}}.zip"
    ///
    /// The following properties are supported:
    /// - {{version}} - the release version in a form of X.Y.Z
    /// - {{os}} - the OS we release for. (`MacOS`, `Linux`, `Windows`, `Android`)
    /// - {{arch}} - the target release architecture. (`x86_64`, `aarch64`)
    #[clap(parse(from_os_str), verbatim_doc_comment)]
    pub release: PathBuf,
    #[clap(long, arg_enum)]
    pub target: Option<PlatformOS>,
}

#[derive(Parser, Debug, Clone)]
pub struct ReleaserOptions {
    /// Specify a releaser version bump strategy
    #[clap(long, default_value = VersionBump::Patch.to_str(), possible_values = VersionBump::variants(), ignore_case = true)]
    pub bump: VersionBump,
}

#[derive(Serialize)]
struct ReleaseInfo {
    version: String,
    os: String,
    arch: String,
}

pub struct Release;

impl Release {
    pub fn new() -> Self {
        Self {}
    }

    fn process_template_path(
        application: &Application,
        path: impl AsRef<Path>,
        target: PlatformOS,
    ) -> PathBuf {
        let new_version = application.image_version();

        let platform = match target {
            PlatformOS::MacOSX8664 | PlatformOS::MacOSAarch64 => "MacOS",
            PlatformOS::WindowsX8664 | PlatformOS::WindowsAarch64 => "Windows",
            PlatformOS::LinuxX8664 | PlatformOS::LinuxAarch64 => "Linux",
            PlatformOS::AndroidAarch64 => "Android",
        };

        let arch = match target {
            PlatformOS::MacOSX8664 => "x86_64",
            PlatformOS::MacOSAarch64 => "aarch64",
            PlatformOS::WindowsX8664 => "x86_64",
            PlatformOS::WindowsAarch64 => "aarch64",
            PlatformOS::LinuxX8664 => "x86_64",
            PlatformOS::LinuxAarch64 => "aarch64",
            PlatformOS::AndroidAarch64 => "aarch64",
        };

        let info = ReleaseInfo {
            version: new_version.to_string(),
            os: platform.to_string(),
            arch: arch.to_string(),
        };

        path.as_ref()
            .iter()
            .map(|each| {
                let template = mustache::compile_str(each.to_str().unwrap()).unwrap();
                template.render_to_string(&info).unwrap()
            })
            .collect::<PathBuf>()
    }

    /// Creates a release package including vm and an image with all extra resources
    /// Platform specific:
    ///  - produces a .zip for desktop targets
    ///  - produces an unsigned .apk
    pub async fn package(
        &self,
        application: &Application,
        release_options: &ReleaseOptions,
    ) -> Result<PathBuf> {
        // resolve an actual target
        let target = release_options
            .target
            .unwrap_or_else(|| application.host_platform());
        // check if the vm for the target exists, and download it otherwise
        if !application.gtoolkit_app_cli_for_target(target).exists() {
            Downloader::new()
                .download_glamorous_toolkit_vm(application, target)
                .await?;
        }

        if target.is_android() {
            return self.create_apk(application, target);
        }

        let package =
            Self::process_template_path(application, release_options.release.as_path(), target);

        ToZip::new(package)
            .one_entry(FileNamed::wildmatch("*.image").within(application.workspace()))
            .one_entry(FileNamed::wildmatch("*.changes").within(application.workspace()))
            .one_entry(FileNamed::wildmatch("*.sources").within(application.workspace()))
            .folder(application.workspace().join("gt-extra"))
            .one_entries(application.gtoolkit_app_entries_for_target(target))
            .zip()
            .map_err(|error| error.into())
    }

    pub async fn run_releaser(
        &self,
        application: &Application,
        releaser_options: &ReleaserOptions,
    ) -> Result<()> {
        SmalltalkCommand::new("releasegtoolkit")
            .arg(format!("--strategy={}", releaser_options.bump.to_str()))
            .arg(format!(
                "--expected={}",
                application.image_version().to_string()
            ))
            .arg(if application.is_verbose() {
                "--verbose"
            } else {
                ""
            })
            .execute(&application.gtoolkit().evaluator())?;

        Ok(())
    }

    fn create_apk(&self, application: &Application, target: PlatformOS) -> Result<PathBuf> {
        use ndk_build::apk::{ApkConfig, StripConfig};
        use ndk_build::manifest::{
            Activity as AndroidActivity, AndroidManifest, Application as AndroidApplication,
            IntentFilter as AndroidIntentFilter, MetaData as AndroidMetaData,
            Permission as AndroidPermission,
        };
        use ndk_build::ndk::Ndk;

        use ndk_build::target::Target as AndroidTarget;

        let android_target = match target {
            PlatformOS::AndroidAarch64 => AndroidTarget::Arm64V8a,
            _ => {
                panic!("Unsupported android target: {:?}", target)
            }
        };

        let manifest_path = application
            .gtoolkit_app_location(target)
            .join("AndroidManifest.xml");

        let manifest_file = File::open(manifest_path.as_path()).unwrap();
        let manifest: AndroidManifest =
            serde_xml_rs::from_reader(BufReader::new(manifest_file)).unwrap();

        println!("manifest: {:#?}", &manifest);

        Ok(manifest_path)

        // let ndk = Ndk::from_env().unwrap();
        // let config = ApkConfig {
        //     ndk: ndk.clone(),
        //     build_dir: bundle_location.clone(),
        //     apk_name: app_name.to_string(),
        //     assets: None,
        //     resources: None,
        //     manifest,
        //     disable_aapt_compression: false,
        //     strip: StripConfig::Default,
        //     reverse_port_forward: Default::default(),
        // };
        //
        // let mut apk = config.create_apk().expect("Create APK");
        // let lib_search_path = self.compiled_libraries_directory(options);
        //
        // self.compiled_libraries(options)
        //     .iter()
        //     .for_each(|compiled_library_path| {
        //         apk.add_lib_recursively(
        //             &compiled_library_path,
        //             android_target,
        //             &[lib_search_path.as_path()],
        //         )
        //         .expect("Add runtime lib")
        //     });
        //
        // let aligned_apk = apk
        //     .add_pending_libs_and_align()
        //     .expect("Add pending libs and align");
    }
}
