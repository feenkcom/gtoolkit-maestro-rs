use crate::gtoolkit::GToolkit;
use crate::options::AppOptions;
use crate::{StartOptions, Starter, BUILDING, CREATING};
use clap::{AppSettings, ArgEnum, Clap};
use feenk_releaser::VersionBump;
use std::str::FromStr;

pub struct Setup;

#[derive(Clap, Debug, Clone)]
#[clap(setting = AppSettings::ColorAlways)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct SetupOptions {
    /// Do not open a default GtWorld
    #[clap(long)]
    pub no_gt_world: bool,
    /// Specify a setup target
    #[clap(long, default_value = "local-build", possible_values = SetupTarget::VARIANTS, case_insensitive = true)]
    pub target: SetupTarget,
    /// When building an image for a release, specify which component version to bump
    #[clap(long, default_value = VersionBump::Patch.to_str(), possible_values = VersionBump::variants(), case_insensitive = true)]
    pub bump: VersionBump,
}

impl SetupOptions {
    pub fn new() -> Self {
        Self {
            no_gt_world: false,
            target: SetupTarget::LocalBuild,
            bump: VersionBump::Patch,
        }
    }

    pub fn target(&mut self, target: SetupTarget) {
        self.target = target;
    }

    pub fn gt_world(&mut self, should_open_gt_world: bool) {
        self.no_gt_world = !should_open_gt_world;
    }

    pub fn bump(&mut self, bump: VersionBump) {
        self.bump = bump;
    }
}

#[derive(ArgEnum, Copy, Clone, Debug)]
#[repr(u32)]
pub enum SetupTarget {
    /// Setup GlamorousToolkit for the local build.
    #[clap(name = "local-build")]
    LocalBuild,
    /// Setup GlamorousToolkit for release
    #[clap(name = "release")]
    Release,
}

impl FromStr for SetupTarget {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        <SetupTarget as ArgEnum>::from_str(s, true)
    }
}

impl ToString for SetupTarget {
    fn to_string(&self) -> String {
        (SetupTarget::VARIANTS[*self as usize]).to_owned()
    }
}

impl Setup {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn setup(
        &self,
        options: &mut AppOptions,
        setup_options: &SetupOptions,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match setup_options.target {
            SetupTarget::LocalBuild => {
                println!("{}Setting up for local build...", CREATING);
                options.gtoolkit().perform_setup_for_local_build()?;
            }
            SetupTarget::Release => {
                println!("{}Setting up for release...", CREATING);
                options
                    .gtoolkit()
                    .perform_setup_for_release(setup_options.bump.clone())?;
                let gtoolkit_version = options.gtoolkit().get_gtoolkit_version()?;
                options.set_gtoolkit_version(gtoolkit_version);
                options.gtoolkit().print_new_commits()?;
            }
        }

        options.gtoolkit().print_vm_version()?;
        options.gtoolkit().print_gtoolkit_version()?;

        if !setup_options.no_gt_world {
            println!("{}Setting up GtWorld...", BUILDING);

            Starter::new()
                .start(options, &StartOptions::default())
                .await?;
        }

        println!("To start GlamorousToolkit run:");
        println!("  cd {:?}", options.workspace());
        println!("  {}", options.gtoolkit_app());

        Ok(())
    }
}
