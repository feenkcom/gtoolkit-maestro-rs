use crate::{
    ExecutableSmalltalk, Result, Smalltalk, SmalltalkCommand, SmalltalkExpression,
    SmalltalkExpressionBuilder, TestOptions,
};
use feenk_releaser::{Version, VersionBump};

pub trait GToolkit {
    fn get_gtoolkit_version(&self) -> Result<Version>;
    fn print_new_commits(&self) -> Result<()>;
    fn perform_setup_for_release(&self, bump: VersionBump) -> Result<()>;
    fn perform_setup_for_local_build(&self) -> Result<()>;
    fn perform_iceberg_clean_up(&self) -> Result<()>;
    fn run_examples(&self, packages: &Vec<String>, test_options: &TestOptions) -> Result<()>;
    fn run_release_examples(&self, test_options: &TestOptions) -> Result<()>;
    fn run_release_slides(&self, test_options: &TestOptions) -> Result<()>;
    fn run_tests(&self, packages: &Vec<String>) -> Result<()>;
    fn run_architectural_report(&self) -> Result<()>;
}

impl<'application> GToolkit for Smalltalk<'application> {
    fn get_gtoolkit_version(&self) -> Result<Version> {
        let version_string =
            SmalltalkCommand::new("getgtoolkitversion").execute_with_result(&self.evaluator())?;
        Version::parse(version_string).map_err(|error| error.into())
    }

    fn print_new_commits(&self) -> Result<()> {
        SmalltalkCommand::new("printNewCommits").execute(&self.evaluator())
    }

    fn perform_setup_for_release(&self, bump: VersionBump) -> Result<()> {
        SmalltalkExpression::new(format!(
            "GtImageSetup performSetupForRelease: '{}'",
            bump.to_str()
        ))
        .execute(self.evaluator().save(true))
    }

    fn perform_setup_for_local_build(&self) -> Result<()> {
        SmalltalkExpression::new("GtImageSetup performLocalSetup")
            .execute(self.evaluator().save(true))
    }

    fn perform_iceberg_clean_up(&self) -> Result<()> {
        SmalltalkExpressionBuilder::new()
            .add("IceCredentialsProvider sshCredentials publicKey: ''; privateKey: ''")
            .add("IceCredentialsProvider useCustomSsh: false")
            .add("IceRepository registry removeAll")
            .add("3 timesRepeat: [ Smalltalk garbageCollect ]")
            .build()
            .execute(self.evaluator().save(true))
    }

    fn run_examples(&self, packages: &Vec<String>, test_options: &TestOptions) -> Result<()> {
        SmalltalkCommand::new("examples")
            .args(packages)
            .arg("--junit-xml-output")
            .arg(if self.verbose() { "--verbose" } else { "" })
            .arg(if test_options.disable_deprecation_rewrites {
                "--disable-deprecation-rewrites"
            } else {
                ""
            })
            .arg(test_options.skip_packages.as_ref().map_or_else(
                || "".to_string(),
                |skip_packages| format!("--skip-packages=\"{}\"", skip_packages.join(",")),
            ))
            .execute(&self.evaluator())
    }

    fn run_release_examples(&self, test_options: &TestOptions) -> Result<()> {
        SmalltalkCommand::new("dedicatedReleaseBranchExamples")
            .arg("--junit-xml-output")
            .arg(if self.verbose() { "--verbose" } else { "" })
            .arg(if test_options.disable_deprecation_rewrites {
                "--disable-deprecation-rewrites"
            } else {
                ""
            })
            .arg(test_options.skip_packages.as_ref().map_or_else(
                || "".to_string(),
                |skip_packages| format!("--skip-packages=\"{}\"", skip_packages.join(",")),
            ))
            .execute(&self.evaluator())
    }

    fn run_release_slides(&self, test_options: &TestOptions) -> Result<()> {
        SmalltalkCommand::new("dedicatedReleaseBranchSlides")
            .arg("--junit-xml-output")
            .arg(if self.verbose() { "--verbose" } else { "" })
            .arg(if test_options.disable_deprecation_rewrites {
                "--disable-deprecation-rewrites"
            } else {
                ""
            })
            .arg(test_options.skip_packages.as_ref().map_or_else(
                || "".to_string(),
                |skip_packages| format!("--skip-packages=\"{}\"", skip_packages.join(",")),
            ))
            .execute(&self.evaluator())
    }

    fn run_tests(&self, packages: &Vec<String>) -> Result<()> {
        SmalltalkCommand::new("test")
            .args(packages)
            .arg("--junit-xml-output")
            .execute(&self.evaluator())
    }

    fn run_architectural_report(&self) -> Result<()> {
        SmalltalkCommand::new("gtexportreport")
            .arg("--report=GtGtoolkitArchitecturalReport")
            .execute(&self.evaluator())
    }
}
