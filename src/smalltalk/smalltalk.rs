use crate::options::AppOptions;
use crate::SmalltalkEvaluator;
use std::error::Error;
use std::path::{Path, PathBuf};

pub trait ExecutableSmalltalk {
    fn execute(&self, evaluator: &SmalltalkEvaluator) -> Result<(), Box<dyn Error>>;

    fn name(&self) -> String;
}

#[derive(Debug, Clone)]
pub struct Smalltalk {
    executable: PathBuf,
    image: PathBuf,
    workspace: Option<PathBuf>,
    options: Option<AppOptions>,
}

impl Smalltalk {
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

    pub fn set_options(self, options: AppOptions) -> Self {
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
        self.options.as_ref()
    }
}
