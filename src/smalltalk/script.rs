use crate::{ExecutableSmalltalk, SmalltalkEvaluator};
use std::error::Error;
use std::path::PathBuf;
use std::process::Command;

pub struct SmalltalkScriptToExecute {
    script: PathBuf,
}

impl SmalltalkScriptToExecute {
    pub fn new(script: impl Into<PathBuf>) -> Self {
        Self {
            script: script.into(),
        }
    }
}

impl ExecutableSmalltalk for SmalltalkScriptToExecute {
    fn create_command(&self, evaluator: &SmalltalkEvaluator) -> Result<Command, Box<dyn Error>> {
        let mut command = evaluator.command();
        command
            .arg("st")
            .arg(if evaluator.should_quit() {
                "--quit"
            } else {
                "--no-quit"
            })
            .arg(if evaluator.should_save() {
                "--save"
            } else {
                ""
            })
            .arg(if evaluator.wants_interactive() {
                "--interactive"
            } else {
                ""
            })
            .arg(self.script.as_path());

        Ok(command)
    }

    fn name(&self) -> String {
        self.script.display().to_string()
    }
}

impl From<SmalltalkScriptToExecute> for Box<(dyn ExecutableSmalltalk + 'static)> {
    fn from(script: SmalltalkScriptToExecute) -> Self {
        Box::new(script)
    }
}
