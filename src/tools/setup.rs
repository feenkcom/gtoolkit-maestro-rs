use crate::create::FileToCreate;
use crate::gtoolkit::GToolkit;
use crate::options::AppOptions;
use crate::{SmalltalkScriptToExecute, SmalltalkScriptsToExecute, BUILDING, CREATING};
use clap::{AppSettings, ArgEnum, Clap};
use std::str::FromStr;

pub struct Setup;

#[derive(Clap, Debug, Clone)]
#[clap(setting = AppSettings::ColorAlways)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct SetupOptions {
    /// Do not open a default GtWorld
    #[clap(long)]
    pub no_gt_world: bool,
    #[clap(long, default_value = "local-build", possible_values = SetupTarget::VARIANTS, case_insensitive = true)]
    /// Specify a setup target
    pub target: SetupTarget,
}

impl SetupOptions {
    pub fn new() -> Self {
        Self {
            no_gt_world: false,
            target: SetupTarget::LocalBuild,
        }
    }

    pub fn target(&mut self, target: SetupTarget) {
        self.target = target;
    }

    pub fn gt_world(&mut self, should_open_gt_world: bool) {
        self.no_gt_world = !should_open_gt_world;
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
        options: &AppOptions,
        setup_options: &SetupOptions,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let gtoolkit = options.gtoolkit();

        match setup_options.target {
            SetupTarget::LocalBuild => {
                println!("{}Setting up for local build...", CREATING);
                gtoolkit.perform_setup_for_local_build()?;
            }
            SetupTarget::Release => {
                println!("{}Setting up for release...", CREATING);
                gtoolkit.perform_setup_for_release()?;
                gtoolkit.print_new_commits()?;
            }
        }

        gtoolkit.print_vm_version()?;

        if !setup_options.no_gt_world {
            println!("{}Setting up GtWorld...", BUILDING);

            FileToCreate::new(
                options.gtoolkit_directory().join("start-gt.st"),
                include_str!("../st/start-gt.st"),
            )
            .create()
            .await?;

            SmalltalkScriptsToExecute::new()
                .add(SmalltalkScriptToExecute::new("start-gt.st"))
                .execute(
                    gtoolkit
                        .evaluator()
                        .save(false)
                        .interactive(true)
                        .quit(false),
                )
                .await?;
        }

        println!("To start GlamorousToolkit run:");
        println!("  cd {:?}", options.gtoolkit_directory());
        println!("  {}", options.gtoolkit_app());

        Ok(())
    }
}