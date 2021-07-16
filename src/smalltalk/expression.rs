use crate::{ExecutableSmalltalk, SmalltalkEvaluator};
use std::error::Error;

pub struct SmalltalkExpression {
    expression: String,
}

impl SmalltalkExpression {
    pub fn new(expression: impl Into<String>) -> Self {
        Self {
            expression: expression.into(),
        }
    }

    pub fn expression(&self) -> &str {
        self.expression.as_str()
    }
}

impl ExecutableSmalltalk for SmalltalkExpression {
    fn execute(&self, evaluator: &SmalltalkEvaluator) -> Result<(), Box<dyn Error>> {
        let expression = if evaluator.should_save() {
            ExpressionBuilder::new()
                .add(&self.expression)
                .add("Smalltalk snapshot: true andQuit: false")
                .build()
                .expression()
                .to_owned()
        } else {
            self.expression.clone()
        };

        let mut command = evaluator.command();
        command
            .arg("eval")
            .arg(if evaluator.should_quit() {
                ""
            } else {
                "--no-quit"
            })
            .arg(if evaluator.wants_interactive() {
                "--interactive"
            } else {
                ""
            })
            .arg(&expression);

        if evaluator.is_verbose() {
            println!("{:?}", &command);
        }

        let status = command.status().unwrap();

        if !status.success() {
            return Err(Box::new(crate::error::Error {
                what: format!(
                    "Expression {} failed. See install.log or install-errors.log for more info",
                    &self.expression
                ),
                source: None,
            }));
        }
        Ok(())
    }

    fn name(&self) -> String {
        self.expression.clone()
    }
}

impl From<SmalltalkExpression> for Box<(dyn ExecutableSmalltalk + 'static)> {
    fn from(expression: SmalltalkExpression) -> Self {
        Box::new(expression)
    }
}

pub struct ExpressionBuilder {
    expressions: Vec<String>,
}

impl ExpressionBuilder {
    pub fn new() -> Self {
        Self {
            expressions: vec![],
        }
    }

    pub fn add(&mut self, expression: impl Into<String>) -> &mut Self {
        self.expressions.push(expression.into());
        self
    }

    pub fn build(&mut self) -> SmalltalkExpression {
        SmalltalkExpression::new(self.expressions.join("."))
    }
}
