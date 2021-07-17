use crate::error::Error;
use crate::options::AppOptions;
use crate::CHECKING;

pub struct Checker;

impl Checker {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn check(
        &self,
        options: &AppOptions,
        should_overwrite: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}Checking the system...", CHECKING);
        if should_overwrite && options.gtoolkit_directory().exists() {
            tokio::fs::remove_dir_all(options.gtoolkit_directory()).await?;
        }

        if options.gtoolkit_directory().exists() {
            return Err(Box::new(Error {
                what: format!(
                    "GToolkit already exists in {:?}",
                    options.gtoolkit_directory().display()
                ),
                source: None,
            }));
        }

        tokio::fs::create_dir_all(options.gtoolkit_directory()).await?;
        Ok(())
    }
}
