use crate::{Application, InstallerError, Result, SmalltalkEvaluator};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub trait ExecutableSmalltalk {
    fn create_command(&self, evaluator: &SmalltalkEvaluator) -> Result<Command>;
    fn execute(&self, evaluator: &SmalltalkEvaluator) -> Result<()> {
        let mut command = self.create_command(evaluator)?;
        if evaluator.is_verbose() {
            println!("{:?}", &command);
        }

        let status = command.status()?;

        if !status.success() {
            return InstallerError::CommandExecutionFailed(command).into();
        }
        Ok(())
    }
    fn execute_with_result(&self, evaluator: &SmalltalkEvaluator) -> Result<String> {
        let mut command = self.create_command(evaluator)?;
        command.stdout(Stdio::piped());

        if evaluator.is_verbose() {
            println!("{:?}", &command);
        }

        let output = command.output()?;

        if !output.status.success() {
            return InstallerError::CommandExecutionFailed(command).into();
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    fn name(&self) -> String;
}

#[derive(Debug, Clone)]
pub struct Smalltalk<'application> {
    executable: PathBuf,
    image: PathBuf,
    application: &'application Application,
}

impl<'application> Smalltalk<'application> {
    pub fn new(
        executable: impl Into<PathBuf>,
        image: impl Into<PathBuf>,
        application: &'application Application,
    ) -> Self {
        Self {
            executable: executable.into(),
            image: image.into(),
            application,
        }
    }

    pub fn executable(&self) -> &Path {
        self.executable.as_path()
    }

    pub fn image(&self) -> &Path {
        self.image.as_path()
    }

    pub fn workspace(&self) -> &Path {
        self.application.workspace()
    }

    pub fn evaluator(&self) -> SmalltalkEvaluator {
        let mut evaluator = SmalltalkEvaluator::new(self);
        evaluator.verbose(self.verbose());
        evaluator
    }

    pub fn verbose(&self) -> bool {
        self.application.is_verbose()
    }

    pub fn application(&self) -> &Application {
        self.application
    }
}
