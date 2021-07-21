use crate::gtoolkit::GToolkit;
use crate::options::AppOptions;
use clap::{AppSettings, Clap};

pub struct Tester;

#[derive(Clap, Debug, Clone)]
#[clap(setting = AppSettings::ColorAlways)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct TestOptions {
    /// Select packages to test. If not specified will run all tests, all slides and architectural reports.
    #[clap(long, min_values = 1)]
    pub packages: Option<Vec<String>>,
}

impl Tester {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn test(
        &self,
        options: &AppOptions,
        test_options: &TestOptions,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let gtoolkit = options.gtoolkit();

        if let Some(ref packages) = test_options.packages {
            gtoolkit.run_examples(packages)?;
        } else {
            gtoolkit.run_release_examples()?;
            gtoolkit.run_release_slides()?;
            gtoolkit.run_architectural_report()?;
        }

        Ok(())
    }
}
