use crate::{Application, InstallerError, Result, CHECKING};

pub struct Checker;

impl Checker {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn check(&self, application: &Application, should_overwrite: bool) -> Result<()> {
        println!("{}Checking the system...", CHECKING);

        if application.image_seed().is_image_file() {
            return Ok(());
        }

        if should_overwrite && application.workspace().exists() {
            tokio::fs::remove_dir_all(application.workspace()).await?;
        }

        if application.workspace().exists() {
            return InstallerError::WorkspaceAlreadyExists(application.workspace().to_path_buf())
                .into();
        }

        tokio::fs::create_dir_all(application.workspace()).await?;
        Ok(())
    }
}
