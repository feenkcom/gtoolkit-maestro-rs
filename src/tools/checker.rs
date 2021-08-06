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
        if should_overwrite && options.workspace().exists() {
            tokio::fs::remove_dir_all(options.workspace()).await?;
        }

        if options.workspace().exists() {
            return Err(Box::new(Error {
                what: format!(
                    "GToolkit already exists in {:?}",
                    options.workspace().display()
                ),
                source: None,
            }));
        }

        tokio::fs::create_dir_all(options.workspace()).await?;
        Ok(())
    }
}
