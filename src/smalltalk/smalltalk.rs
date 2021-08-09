use crate::options::AppOptions;
use crate::SmalltalkEvaluator;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub trait ExecutableSmalltalk {
    fn create_command(&self, evaluator: &SmalltalkEvaluator) -> Result<Command, Box<dyn Error>>;
    fn execute(&self, evaluator: &SmalltalkEvaluator) -> Result<(), Box<dyn Error>> {
        let mut command = self.create_command(evaluator)?;
        if evaluator.is_verbose() {
            println!("{:?}", &command);
        }

        let status = command.status()?;

        if !status.success() {
            return Err(Box::new(crate::error::Error {
                what: format!(
                    "Command {:?} failed. See install.log or install-errors.log for more info",
                    &command
                ),
                source: None,
            }));
        }
        Ok(())
    }
    fn execute_with_result(
        &self,
        evaluator: &SmalltalkEvaluator,
    ) -> Result<String, Box<dyn Error>> {
        let mut command = self.create_command(evaluator)?;
        command.stdout(Stdio::piped());

        if evaluator.is_verbose() {
            println!("{:?}", &command);
        }

        let output = command.output()?;

        if !output.status.success() {
            return Err(Box::new(crate::error::Error {
                what: format!(
                    "Command {:?} failed.\nError:\n{}",
                    &command,
                    String::from_utf8_lossy(&output.stderr)
                ),
                source: None,
            }));
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    fn name(&self) -> String;
}

#[derive(Debug, Clone)]
pub struct Smalltalk<'options> {
    executable: PathBuf,
    image: PathBuf,
    workspace: Option<PathBuf>,
    options: Option<&'options AppOptions>,
}

impl<'options> Smalltalk<'options> {
    pub fn new(executable: impl Into<PathBuf>, image: impl Into<PathBuf>) -> Self {
        Self {
            executable: executable.into(),
            image: image.into(),
            workspace: None,
            options: None,
        }
    }

    pub fn executable(&self) -> &Path {
        self.executable.as_path()
    }

    pub fn image(&self) -> &Path {
        self.image.as_path()
    }

    pub fn workspace(&self) -> PathBuf {
        self.workspace
            .as_ref()
            .map_or_else(|| std::env::current_dir().unwrap(), |path| path.clone())
    }

    pub fn set_workspace(self, workspace: impl Into<PathBuf>) -> Self {
        Self {
            executable: self.executable,
            image: self.image,
            workspace: Some(workspace.into()),
            options: self.options,
        }
    }

    pub fn set_options(self, options: &'options AppOptions) -> Self {
        Self {
            executable: self.executable,
            image: self.image,
            workspace: self.workspace,
            options: Some(options),
        }
    }

    pub fn evaluator(&self) -> SmalltalkEvaluator {
        let mut evaluator = SmalltalkEvaluator::new(self);
        if let Some(ref options) = self.options {
            evaluator.verbose(options.verbose());
        }
        evaluator
    }

    pub fn options(&self) -> Option<&AppOptions> {
        self.options
    }

    pub fn verbose(&self) -> bool {
        self.options
            .as_ref()
            .map_or(false, |options| options.verbose())
    }
}
