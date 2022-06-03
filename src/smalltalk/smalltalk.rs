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
    flags: SmalltalkFlags,
}

impl<'application> Smalltalk<'application> {
    pub fn new(
        executable: impl Into<PathBuf>,
        image: impl Into<PathBuf>,
        flags: SmalltalkFlags,
        application: &'application Application,
    ) -> Self {
        Self {
            executable: executable.into(),
            image: image.into(),
            application,
            flags,
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

    pub fn flags(&self) -> &SmalltalkFlags {
        &self.flags
    }

    pub fn verbose(&self) -> bool {
        self.application.is_verbose()
    }

    pub fn application(&self) -> &Application {
        self.application
    }
}

#[derive(Debug, Clone)]
pub struct SmalltalkFlags {
    interactive: Option<String>,
    headless: Option<String>,
}

impl SmalltalkFlags {
    pub fn pharo() -> Self {
        Self {
            interactive: None,
            headless: Some("--headless".to_string()),
        }
    }

    pub fn gtoolkit() -> Self {
        Self {
            interactive: Some("--interactive".to_string()),
            headless: None,
        }
    }

    pub fn interactive_or_headless_flag(&self, is_interactive: bool) -> Option<&str> {
        if is_interactive {
            self.interactive.as_ref().map(|flag| flag.as_str())
        } else {
            self.headless.as_ref().map(|flag| flag.as_str())
        }
    }
}
