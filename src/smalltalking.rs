use indicatif::{ProgressBar, ProgressStyle};
use std::error::Error;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub struct SmalltalkScriptsToExecute {
    workspace: PathBuf,
    scripts: Vec<SmalltalkScriptToExecute>,
}

pub struct SmalltalkScriptToExecute {
    executable: PathBuf,
    image: PathBuf,
    script: PathBuf,
    should_quit: bool,
    interactive: bool,
}

impl SmalltalkScriptToExecute {
    pub fn new(executable: PathBuf, image: PathBuf, script: impl Into<PathBuf>) -> Self {
        Self {
            executable,
            image,
            script: script.into(),
            should_quit: true,
            interactive: false,
        }
    }

    pub fn no_quit(self) -> Self {
        Self {
            executable: self.executable,
            image: self.image,
            script: self.script,
            should_quit: false,
            interactive: self.interactive,
        }
    }
    pub fn interactive(self) -> Self {
        Self {
            executable: self.executable,
            image: self.image,
            script: self.script,
            should_quit: self.should_quit,
            interactive: true,
        }
    }
}

impl SmalltalkScriptsToExecute {
    pub fn new(workspace: impl Into<PathBuf>) -> Self {
        Self {
            workspace: workspace.into(),
            scripts: vec![],
        }
    }

    pub fn add(self, script: SmalltalkScriptToExecute) -> Self {
        let mut scripts = self.scripts;
        scripts.push(script);
        Self {
            workspace: self.workspace,
            scripts,
        }
    }

    pub async fn execute(&self) -> Result<(), Box<dyn Error>> {
        let mut index = 0 as usize;
        let total = self.scripts.len();
        for script in &self.scripts {
            index += 1;
            let pb = ProgressBar::new_spinner();
            pb.enable_steady_tick(120);
            pb.set_style(
                ProgressStyle::default_spinner()
                    .tick_strings(&[
                        "🌑 ", "🌒 ", "🌓 ", "🌔 ", "🌕 ", "🌖 ", "🌗 ", "🌘 ", "✅ ",
                    ])
                    .template("{prefix:.bold.dim} {spinner:.blue} {wide_msg}"),
            );
            pb.set_message(format!("Executing {:?}", &script.script.display()));
            pb.set_prefix(format!("[{}/{}]", index, total));

            let status = Command::new(&script.executable)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .current_dir(&self.workspace)
                .arg(&script.image)
                .arg("st")
                .arg(if script.should_quit {
                    "--quit"
                } else {
                    "--no-quit"
                })
                .arg(if script.interactive {
                    "--interactive"
                } else {
                    ""
                })
                .arg(&script.script)
                .status()
                .unwrap();
            pb.finish_with_message(format!("Finished {:?}", &script.script.display()));

            if !status.success() {
                return Err(Box::new(crate::error::Error {
                    what: format!(
                        "Script {} failed. See PharoDebug.log or crash.dmp for more info",
                        &script.script.display()
                    ),
                    source: None,
                }));
            }
        }

        Ok(())
    }
}
