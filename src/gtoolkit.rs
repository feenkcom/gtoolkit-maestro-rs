use crate::{
    ExecutableSmalltalk, Smalltalk, SmalltalkCommand, SmalltalkExpression,
    SmalltalkExpressionBuilder,
};
use std::error::Error;
use std::fs::File;
use std::io::Write;

pub trait GToolkit {
    fn set_deprecation_rewrites(&self, enabled: bool) -> Result<(), Box<dyn Error>>;
    fn disable_deprecation_rewrites(&self) -> Result<(), Box<dyn Error>> {
        self.set_deprecation_rewrites(false)
    }
    fn enable_deprecation_rewrites(&self) -> Result<(), Box<dyn Error>> {
        self.set_deprecation_rewrites(true)
    }
    fn print_vm_version(&self) -> Result<(), Box<dyn Error>>;
    fn print_gtoolkit_version(&self) -> Result<(), Box<dyn Error>>;
    fn print_new_commits(&self) -> Result<(), Box<dyn Error>>;
    fn perform_setup_for_release(&self) -> Result<(), Box<dyn Error>>;
    fn perform_setup_for_local_build(&self) -> Result<(), Box<dyn Error>>;
    fn perform_iceberg_clean_up(&self) -> Result<(), Box<dyn Error>>;
    fn run_examples(&self, packages: &Vec<String>) -> Result<(), Box<dyn Error>>;
    fn run_release_examples(&self) -> Result<(), Box<dyn Error>>;
    fn run_release_slides(&self) -> Result<(), Box<dyn Error>>;
    fn run_architectural_report(&self) -> Result<(), Box<dyn Error>>;
}

impl GToolkit for Smalltalk {
    fn set_deprecation_rewrites(&self, enabled: bool) -> Result<(), Box<dyn Error>> {
        SmalltalkExpression::new(format!("Deprecation activateTransformations: {}", enabled))
            .execute(self.evaluator().save(true))
    }

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

    fn print_new_commits(&self) -> Result<(), Box<dyn Error>> {
        SmalltalkCommand::new("printNewCommits").execute(&self.evaluator())
    }

    fn perform_setup_for_release(&self) -> Result<(), Box<dyn Error>> {
        SmalltalkExpression::new("GtImageSetup performSetupForRelease")
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

    fn run_examples(&self, packages: &Vec<String>) -> Result<(), Box<dyn Error>> {
        SmalltalkCommand::new("examples")
            .args(packages)
            .arg("--junit-xml-output")
            .arg(if self.verbose() { "--verbose" } else { "" })
            .execute(&self.evaluator())
    }

    fn run_release_examples(&self) -> Result<(), Box<dyn Error>> {
        SmalltalkCommand::new("dedicatedReleaseBranchExamples")
            .arg("--junit-xml-output")
            .arg(if self.verbose() { "--verbose" } else { "" })
            .execute(&self.evaluator())
    }

    fn run_release_slides(&self) -> Result<(), Box<dyn Error>> {
        SmalltalkCommand::new("dedicatedReleaseBranchSlides")
            .arg("--junit-xml-output")
            .arg(if self.verbose() { "--verbose" } else { "" })
            .execute(&self.evaluator())
    }

    fn run_architectural_report(&self) -> Result<(), Box<dyn Error>> {
        SmalltalkCommand::new("gtexportreport")
            .arg("--report=GtGtoolkitArchitecturalReport")
            .execute(&self.evaluator())
    }
}
