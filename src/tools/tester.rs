use crate::gtoolkit::GToolkit;
use crate::options::AppOptions;

pub struct Tester;

impl Tester {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn test_release(
        &self,
        options: &AppOptions,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let gtoolkit = options.gtoolkit();

        gtoolkit.run_release_examples()?;
        gtoolkit.run_release_slides()?;
        gtoolkit.run_architectural_report()?;

        Ok(())
    }
}
