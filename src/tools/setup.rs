use crate::gtoolkit::GToolkit;
use crate::{Application, Result, StartOptions, Starter, BUILDING, CREATING};
use clap::{ArgEnum, Parser};
use feenk_releaser::VersionBump;
use std::str::FromStr;

pub struct Setup;

#[derive(Parser, Debug, Clone)]
pub struct SetupOptions {
    /// Do not open a default GtWorld
    #[clap(long)]
    pub no_gt_world: bool,
    /// Specify a setup target
    #[clap(long, default_value = "local-build", arg_enum, ignore_case = true)]
    pub target: SetupTarget,
    /// When building an image for a release, specify which component version to bump
    #[clap(long, default_value = VersionBump::Patch.to_str(), possible_values = VersionBump::variants(), ignore_case = true)]
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

    pub fn setup_target(&mut self, target: SetupTarget) {
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

    fn from_str(s: &str) -> std::result::Result<Self, String> {
        <SetupTarget as ArgEnum>::from_str(s, true)
    }
}

impl ToString for SetupTarget {
    fn to_string(&self) -> String {
        self.to_possible_value().unwrap().get_name().to_string()
    }
}

impl Setup {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn setup(
        &self,
        application: &mut Application,
        setup_options: &SetupOptions,
    ) -> Result<()> {
        match setup_options.target {
            SetupTarget::LocalBuild => {
                println!("{}Setting up for local build...", CREATING);
                application.gtoolkit().perform_setup_for_local_build()?;
            }
            SetupTarget::Release => {
                println!("{}Setting up for release...", CREATING);
                application
                    .gtoolkit()
                    .perform_setup_for_release(setup_options.bump.clone())?;
                let gtoolkit_version = application.gtoolkit().get_gtoolkit_version()?;
                application.set_image_version(gtoolkit_version.into());
                application.gtoolkit().print_new_commits()?;
            }
        }

        application.serialize_into_file()?;

        if !setup_options.no_gt_world {
            println!("{}Setting up GtWorld...", BUILDING);

            Starter::new()
                .start(application, &StartOptions::default())
                .await?;
        }

        println!("To start GlamorousToolkit run:");
        println!("  cd {:?}", application.workspace());
        println!("  {}", application.gtoolkit_app());

        Ok(())
    }
}
