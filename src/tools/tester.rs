use crate::gtoolkit::GToolkit;
use crate::Application;
use crate::Result;
use clap::Parser;

pub struct Tester;

#[derive(Parser, Debug, Clone)]
pub struct TestOptions {
    /// Select packages to test. If not specified will run all tests, all slides and architectural reports.
    #[clap(long, min_values = 1)]
    pub packages: Option<Vec<String>>,
    /// Disable automatic deprecation rewrites during testing phase
    #[clap(long)]
    pub disable_deprecation_rewrites: bool,
    /// Disable running Pharo's TestCase when packages are provided. Please note that Pharo's `test` runner does not support skipping packages
    #[clap(long)]
    pub disable_tests: bool,
    #[clap(long, min_values = 1)]
    pub skip_packages: Option<Vec<String>>,
    /// Run the image with a windowing event loop rather than headless.
    ///
    /// If enabled, a `RustWindowingEventFetcher` loop is started in GToolkit.
    #[clap(long)]
    pub interactive: bool,
    /// Run the full release example suite with parallel Smalltalk workers.
    #[clap(long, conflicts_with = "packages")]
    pub parallel_examples: bool,
}

impl Tester {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn test(&self, application: &Application, test_options: &TestOptions) -> Result<()> {
        let gtoolkit = application.gtoolkit();

        if let Some(ref packages) = test_options.packages {
            gtoolkit.run_examples(packages, test_options)?;
            if !test_options.disable_tests {
                gtoolkit.run_tests(packages, test_options)?;
            }
        } else {
            if test_options.parallel_examples {
                gtoolkit.run_parallel_release_examples(test_options)?;
            } else {
                gtoolkit.run_release_examples(test_options)?;
            }
            gtoolkit.run_release_slides(test_options)?;
            gtoolkit.run_architectural_report()?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::TestOptions;
    use clap::Parser;

    #[test]
    fn parallel_examples_only_accepts_release_suite() {
        let options = TestOptions::try_parse_from(["test", "--parallel-examples"])
            .expect("parallel release option should parse");
        assert!(options.parallel_examples);
        assert!(TestOptions::try_parse_from([
            "test",
            "--parallel-examples",
            "--packages",
            "GToolkit-Editor",
        ])
        .is_err());
    }
}
