use crate::{ExecutableSmalltalk, SmalltalkEvaluator};
use std::error::Error;

pub struct SmalltalkCommand {
    command: String,
    arguments: Vec<String>,
}

impl SmalltalkCommand {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            arguments: vec![],
        }
    }

    pub fn arg(self, arg: impl Into<String>) -> Self {
        let arg = arg.into();
        if arg.is_empty() {
            return self;
        };

        let mut args = self.arguments;
        args.push(arg);
        Self {
            command: self.command,
            arguments: args,
        }
    }

    pub fn args(self, args: &Vec<impl AsRef<str>>) -> Self {
        let mut command = self;
        for arg in args {
            command = command.arg(arg.as_ref().to_owned());
        }
        command
    }
}

impl ExecutableSmalltalk for SmalltalkCommand {
    fn execute(&self, evaluator: &SmalltalkEvaluator) -> Result<(), Box<dyn Error>> {
        let mut command = evaluator.command();
        command.arg(&self.command);
        command.args(&self.arguments);
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
