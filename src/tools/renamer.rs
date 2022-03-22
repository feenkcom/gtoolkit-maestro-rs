use crate::{Application, ExecutableSmalltalk, ImageSeed, Result, SmalltalkCommand};
use clap::Parser;

pub struct Renamer;

#[derive(Parser, Debug, Clone)]
pub struct RenameOptions {
    /// A new name of the image without the extension
    pub name: String,
}

impl Renamer {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn rename(
        &self,
        application: &mut Application,
        rename_options: &RenameOptions,
    ) -> Result<()> {
        let current_image_path = application.image();
        let current_changes_file = current_image_path.with_extension("changes");

        let new_image_path =
            current_image_path.with_file_name(format!("{}.image", rename_options.name.as_str()));

        SmalltalkCommand::new("save")
            .arg(rename_options.name.as_str())
            .arg("--delete-old")
            .execute(application.gtoolkit().evaluator().save(true))?;

        if current_changes_file.exists() {
            std::fs::remove_file(current_changes_file)?;
        }

        application.set_image_seed(ImageSeed::Image(new_image_path))?;
        application.serialize_into_file()?;

        Ok(())
    }
}
