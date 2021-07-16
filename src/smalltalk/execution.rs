use crate::{ExecutableSmalltalk, SmalltalkEvaluator};
use indicatif::{ProgressBar, ProgressStyle};
use std::error::Error;

pub struct SmalltalkScriptsToExecute {
    scripts: Vec<Box<dyn ExecutableSmalltalk>>,
}

impl SmalltalkScriptsToExecute {
    pub fn new() -> Self {
        Self { scripts: vec![] }
    }

    pub fn add(&mut self, script: impl Into<Box<dyn ExecutableSmalltalk>>) -> &mut Self {
        self.scripts.push(script.into());
        self
    }

    pub async fn execute(&self, evaluator: &SmalltalkEvaluator<'_>) -> Result<(), Box<dyn Error>> {
        let mut index = 0 as usize;
        let total = self.scripts.len();

        for script in &self.scripts {
            index += 1;
            let pb = if evaluator.is_verbose() {
                println!("[{}/{}] Executing {:?}", index, total, script.name());
                None
            } else {
                let pb = ProgressBar::new_spinner();

                pb.enable_steady_tick(120);
                pb.set_style(
                    ProgressStyle::default_spinner()
                        .tick_strings(&[
                            "🌑 ", "🌒 ", "🌓 ", "🌔 ", "🌕 ", "🌖 ", "🌗 ", "🌘 ", "✅ ",
                        ])
                        .template("{prefix:.bold.dim} {spinner:.blue} {wide_msg}"),
                );
                pb.set_message(format!("Executing {:?}", script.name()));
                pb.set_prefix(format!("[{}/{}]", index, total));

                Some(pb)
            };

            script.execute(evaluator)?;

            if let Some(ref pb) = pb {
                pb.finish_with_message(format!("Finished {:?}", script.name()));
            } else {
                println!("Finished {:?}", script.name());
            }
        }

        Ok(())
    }
}
