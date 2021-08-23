use crate::gtoolkit::GToolkit;
use crate::{Application, Result};

pub struct Cleaner;

impl Cleaner {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn clean(&self, application: &Application) -> Result<()> {
        application.gtoolkit().perform_iceberg_clean_up()?;

        Ok(())
    }
}
