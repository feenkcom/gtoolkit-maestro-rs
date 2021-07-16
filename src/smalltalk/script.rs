use crate::{ExecutableSmalltalk, SmalltalkEvaluator};
use std::error::Error;
use std::path::PathBuf;

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
    fn execute(&self, evaluator: &SmalltalkEvaluator) -> Result<(), Box<dyn Error>> {
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

        if evaluator.is_verbose() {
            println!("{:?}", &command);
        }
        let status = command.status().unwrap();

        if !status.success() {
            return Err(Box::new(crate::error::Error {
                what: format!(
                    "Script {} failed. See install.log or install-errors.log for more info",
                    self.script.display()
                ),
                source: None,
            }));
        }
        Ok(())
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
