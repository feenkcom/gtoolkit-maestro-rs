use crate::{InstallerError, Result, Smalltalk};
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug)]
pub struct SmalltalkEvaluator<'smalltalk, 'options> {
    smalltalk: &'smalltalk Smalltalk<'options>,
    interactive: bool,
    should_quit: bool,
    should_save: bool,
    verbose: bool,
    use_image: bool,
}

impl<'smalltalk, 'options> SmalltalkEvaluator<'smalltalk, 'options> {
    pub fn new(smalltalk: &'smalltalk Smalltalk<'options>) -> Self {
        Self {
            smalltalk,
            interactive: false,
            should_quit: true,
            should_save: false,
            verbose: false,
            use_image: true,
        }
    }

    pub fn interactive(&mut self, interactive: bool) -> &mut Self {
        self.interactive = interactive;
        self
    }

    pub fn quit(&mut self, should_quit: bool) -> &mut Self {
        self.should_quit = should_quit;
        self
    }

    pub fn save(&mut self, should_save: bool) -> &mut Self {
        self.should_save = should_save;
        self
    }

    pub fn verbose(&mut self, verbose: bool) -> &mut Self {
        self.verbose = verbose;
        self
    }

    pub fn without_image(&mut self) -> &mut Self {
        self.use_image = false;
        self
    }

    pub fn workspace(&self) -> PathBuf {
        let app_workspace = self.smalltalk.workspace();
        if !app_workspace.exists() {
            return PathBuf::from(".");
        }
        app_workspace.to_path_buf()
    }

    pub fn executable(&self) -> &Path {
        self.smalltalk.executable()
    }

    pub fn image(&self) -> &Path {
        self.smalltalk.image()
    }

    pub fn should_save(&self) -> bool {
        self.should_save
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn wants_interactive(&self) -> bool {
        self.interactive
    }

    pub fn interactive_or_headless_flag(&self) -> Option<&str> {
        self.smalltalk
            .flags()
            .interactive_or_headless_flag(self.wants_interactive())
    }

    pub fn is_verbose(&self) -> bool {
        self.verbose
    }

    pub fn stdout(&self) -> Stdio {
        if self.is_verbose() {
            return Stdio::inherit();
        }

        let stdout = OpenOptions::new()
            .append(true)
            .write(true)
            .create(true)
            .open(self.workspace().join("install.log"))
            .unwrap();

        Stdio::from(stdout)
    }

    pub fn stderr(&self) -> Stdio {
        if self.is_verbose() {
            return Stdio::inherit();
        }

        let stderr = OpenOptions::new()
            .append(true)
            .write(true)
            .create(true)
            .open(self.workspace().join("install-errors.log"))
            .unwrap();

        Stdio::from(stderr)
    }

    pub fn command(&self) -> Result<Command> {
        let relative_executable = self.workspace().join(self.executable());
        let executable = to_absolute::canonicalize(&relative_executable)
            .map_err(|error| InstallerError::CanonicalizeError(relative_executable, error))?;

        let mut command = Command::new(executable);
        command
            .current_dir(self.workspace())
            .stdout(self.stdout())
            .stderr(self.stderr());

        if let Some(flag) = self.interactive_or_headless_flag() {
            command.arg(flag);
        }
        if self.use_image {
            command.arg(self.image());
        }
        Ok(command)
    }
}
