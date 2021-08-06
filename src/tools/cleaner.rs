use crate::gtoolkit::GToolkit;
use crate::options::AppOptions;

pub struct Cleaner;

impl Cleaner {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn clean(&self, options: &AppOptions) -> Result<(), Box<dyn std::error::Error>> {
        options.gtoolkit().perform_iceberg_clean_up()?;

        Ok(())
    }
}
