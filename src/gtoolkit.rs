use crate::{
    ExecutableSmalltalk, Smalltalk, SmalltalkCommand, SmalltalkExpression,
    SmalltalkExpressionBuilder, TestOptions,
};
use feenk_releaser::{Version, VersionBump};
use std::error::Error;
use std::fs::File;
use std::io::Write;

pub trait GToolkit {
    fn print_vm_version(&self) -> Result<(), Box<dyn Error>>;
    fn print_gtoolkit_version(&self) -> Result<(), Box<dyn Error>>;
    fn get_gtoolkit_version(&self) -> Result<Version, Box<dyn Error>>;
    fn print_new_commits(&self) -> Result<(), Box<dyn Error>>;
    fn perform_setup_for_release(&self, bump: VersionBump) -> Result<(), Box<dyn Error>>;
    fn perform_setup_for_local_build(&self) -> Result<(), Box<dyn Error>>;
    fn perform_iceberg_clean_up(&self) -> Result<(), Box<dyn Error>>;
    fn run_examples(
        &self,
        packages: &Vec<String>,
        test_options: &TestOptions,
    ) -> Result<(), Box<dyn Error>>;
    fn run_release_examples(&self, test_options: &TestOptions) -> Result<(), Box<dyn Error>>;
    fn run_release_slides(&self, test_options: &TestOptions) -> Result<(), Box<dyn Error>>;
    fn run_architectural_report(&self) -> Result<(), Box<dyn Error>>;
}

impl GToolkit for Smalltalk {
    fn print_vm_version(&self) -> Result<(), Box<dyn Error>> {
        let options = self.options().expect("Options are not set");
        let version = options.vm_version().expect("VM version is not set");

        let mut file = File::create(options.vm_version_file()).expect("Could not create file");
        file.write_fmt(format_args!("v{}", version))?;
        Ok(())
    }

    fn print_gtoolkit_version(&self) -> Result<(), Box<dyn Error>> {
        let options = self.options().expect("Options are not set");
        let version = options
            .gtoolkit_version()
            .expect("GToolkit version is not set");

        let mut file =
            File::create(options.gtoolkit_version_file()).expect("Could not create file");
        file.write_fmt(format_args!("v{}", version))?;
        Ok(())
    }

    fn get_gtoolkit_version(&self) -> Result<Version, Box<dyn Error>> {
        let version_string =
            SmalltalkCommand::new("getgtoolkitversion").execute_with_result(&self.evaluator())?;
        Version::parse(version_string)
    }

    fn print_new_commits(&self) -> Result<(), Box<dyn Error>> {
        SmalltalkCommand::new("printNewCommits").execute(&self.evaluator())
    }

    fn perform_setup_for_release(&self, bump: VersionBump) -> Result<(), Box<dyn Error>> {
        SmalltalkExpression::new(format!(
            "GtImageSetup performSetupForRelease: '{}'",
            bump.to_str()
        ))
        .execute(self.evaluator().save(true))
    }

    fn perform_setup_for_local_build(&self) -> Result<(), Box<dyn Error>> {
        SmalltalkExpression::new("GtImageSetup performLocalSetup")
            .execute(self.evaluator().save(true))
    }

    fn perform_iceberg_clean_up(&self) -> Result<(), Box<dyn Error>> {
        SmalltalkExpressionBuilder::new()
            .add("IceCredentialsProvider sshCredentials publicKey: ''; privateKey: ''")
            .add("IceCredentialsProvider useCustomSsh: false")
            .add("IceRepository registry removeAll")
            .add("3 timesRepeat: [ Smalltalk garbageCollect ]")
            .build()
            .execute(self.evaluator().save(true))
    }

    fn run_examples(
        &self,
        packages: &Vec<String>,
        test_options: &TestOptions,
    ) -> Result<(), Box<dyn Error>> {
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

    fn run_release_examples(&self, test_options: &TestOptions) -> Result<(), Box<dyn Error>> {
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

    fn run_release_slides(&self, test_options: &TestOptions) -> Result<(), Box<dyn Error>> {
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

    fn run_architectural_report(&self) -> Result<(), Box<dyn Error>> {
        SmalltalkCommand::new("gtexportreport")
            .arg("--report=GtGtoolkitArchitecturalReport")
            .execute(&self.evaluator())
    }
}
