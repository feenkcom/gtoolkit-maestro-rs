use crate::{ExecutableSmalltalk, Result, SmalltalkEvaluator};
use std::ffi::OsString;
use std::process::Command;

pub struct SmalltalkCommand {
    command: String,
    arguments: Vec<OsString>,
}

impl SmalltalkCommand {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            arguments: vec![],
        }
    }

    pub fn arg(self, arg: impl Into<OsString>) -> Self {
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
    fn create_command(&self, evaluator: &SmalltalkEvaluator) -> Result<Command> {
        let mut command = evaluator.command()?;
        command.arg(&self.command);
        command.args(&self.arguments);

        Ok(command)
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
