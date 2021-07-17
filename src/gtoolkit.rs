use crate::{ExecutableSmalltalk, Smalltalk, SmalltalkCommand, SmalltalkExpression};
use std::error::Error;
use std::fs::File;
use std::io::Write;

pub trait GToolkit {
    fn print_vm_version(&self) -> Result<(), Box<dyn Error>>;
    fn print_new_commits(&self) -> Result<(), Box<dyn Error>>;
    fn perform_setup_for_release(&self) -> Result<(), Box<dyn Error>>;
    fn perform_setup_for_local_build(&self) -> Result<(), Box<dyn Error>>;
    fn run_release_examples(&self) -> Result<(), Box<dyn Error>>;
    fn run_release_slides(&self) -> Result<(), Box<dyn Error>>;
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
