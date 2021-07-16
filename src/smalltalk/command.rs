use crate::{ExecutableSmalltalk, SmalltalkEvaluator};
use std::error::Error;

pub struct SmalltalkCommand {
    command: String,
}

impl SmalltalkCommand {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
        }
    }
}

impl ExecutableSmalltalk for SmalltalkCommand {
    fn execute(&self, evaluator: &SmalltalkEvaluator) -> Result<(), Box<dyn Error>> {
        let mut command = evaluator.command();
        command.arg(&self.command);
        if evaluator.is_verbose() {
            println!("{:?}", &command);
        }

        let status = command.status().unwrap();

        if !status.success() {
            return Err(Box::new(crate::error::Error {
                what: format!(
                    "Command {} failed. See install.log or install-errors.log for more info",
                    &self.command
                ),
                source: None,
            }));
        }
        Ok(())
    }

    fn name(&self) -> String {
        self.command.clone()
    }
}

impl From<SmalltalkCommand> for Box<(dyn ExecutableSmalltalk + 'static)> {
    fn from(command: SmalltalkCommand) -> Self {
        Box::new(command)
    }
}
