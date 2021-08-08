use clap::{AppSettings, Clap};
use parse_duration::parse as duration_parse;
use std::time::Duration;

use crate::options::AppOptions;
use crate::{ExecutableSmalltalk, SmalltalkExpressionBuilder};

const DEFAULT_APPLICATION_STARTER: &str = "GtWorld openDefault";
const DEFAULT_DELAY: &str = "5 seconds";

#[derive(Clap, Debug, Clone)]
#[clap(setting = AppSettings::ColorAlways)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct StartOptions {
    /// An amount of time to wait before saving and closing an app
    #[clap(long, parse(try_from_str = duration_parse), default_value = DEFAULT_DELAY)]
    pub delay: Duration,
    /// A Smalltalk expression that starts an application
    #[clap(long, default_value = DEFAULT_APPLICATION_STARTER)]
    pub expression: String,
}

impl Default for StartOptions {
    fn default() -> Self {
        Self {
            delay: duration_parse(DEFAULT_DELAY).expect("failed to parse default duration"),
            expression: DEFAULT_APPLICATION_STARTER.to_owned(),
        }
    }
}

pub struct Starter;

impl Starter {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn start(
        &self,
        options: &AppOptions,
        start_options: &StartOptions,
    ) -> Result<(), Box<dyn std::error::Error>> {
        SmalltalkExpressionBuilder::new()
            .add(&start_options.expression)
            .add(format!(
                "{} milliSeconds wait",
                &start_options.delay.as_millis()
            ))
            .add("BlHost pickHost universe snapshot: true andQuit: true")
            .build()
            .execute(
                options
                    .gtoolkit()
                    .evaluator()
                    .save(false)
                    .interactive(true)
                    .quit(false),
            )?;
        Ok(())
    }
}
