use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Default)]
pub struct Scripts {
    on_pre_build: Vec<PathBuf>,
    on_post_build: Vec<PathBuf>,
}

pub enum ScriptError {
    CouldNotExecute(String),
    ScriptFailure(String, i32, String, String),
}

impl Scripts {
    pub fn from_directory(dir: impl AsRef<Path>) -> Self {
        if let Ok(dir) = read_dir(dir.as_ref()) {
            let mut scripts = Self::default();

            for entry in dir.flatten() {
                if entry.path().is_file() {
                    if entry
                        .file_name()
                        .to_string_lossy()
                        .starts_with("onPreBuild")
                    {
                        scripts.on_pre_build.push(entry.path());
                    } else if entry
                        .file_name()
                        .to_string_lossy()
                        .starts_with("onPostBuild")
                    {
                        scripts.on_post_build.push(entry.path());
                    }
                }
            }

            scripts
        } else {
            Self::default()
        }
    }

    pub fn execute_pre_build(&self) -> Result<(), ScriptError> {
        self.execute(&self.on_pre_build)
    }

    pub fn execute_post_build(&self) -> Result<(), ScriptError> {
        self.execute(&self.on_post_build)
    }

    fn execute(&self, scripts: &[PathBuf]) -> Result<(), ScriptError> {
        for script in scripts {
            log!(
                "Executing",
                "script `{}`",
                script.file_name().unwrap().to_string_lossy()
            );

            let output = Command::new(script).output().map_err(|_| {
                ScriptError::CouldNotExecute(
                    script.file_name().unwrap().to_string_lossy().to_string(),
                )
            })?;

            if !output.status.success() {
                return Err(ScriptError::ScriptFailure(
                    script.file_name().unwrap().to_string_lossy().to_string(),
                    output.status.code().unwrap_or(-1),
                    String::from_utf8_lossy(&output.stdout).trim().to_string(),
                    String::from_utf8_lossy(&output.stderr).trim().to_string(),
                ));
            }
        }

        Ok(())
    }
}
