use crate::create::FileToCreate;
use crate::options::AppOptions;
use crate::smalltalking::{SmalltalkScriptToExecute, SmalltalkScriptsToExecute};
use crate::{BUILDING, CREATING};

pub struct Setup;

impl Setup {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn start(&self, options: &AppOptions) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}Creating starter scripts...", CREATING);
        FileToCreate::new(
            options.gtoolkit_directory().join("start-gt.st"),
            include_str!("../st/start-gt.st"),
        )
        .create()
        .await?;

        println!("{}Starting the Glamorous Toolkit...", BUILDING);
        SmalltalkScriptsToExecute::new(options.gtoolkit_directory())
            .add(
                SmalltalkScriptToExecute::new(
                    options.gtoolkit_app_cli(),
                    options.gtoolkit_image(),
                    "start-gt.st",
                )
                .no_quit()
                .interactive(),
            )
            .execute()
            .await?;

        println!("To start GlamorousToolkit run:");
        println!("  cd {:?}", options.gtoolkit_directory());
        println!("  {}", options.gtoolkit_app());

        Ok(())
    }
}
