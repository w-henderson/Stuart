//! Provides methods for locating and executing build scripts.

use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Defines constant values, specific to the OS.
#[cfg(target_os = "windows")]
mod constants {
    /// The names of scripts to run before building.
    pub(super) static PRE_BUILD_SCRIPT_NAMES: [&str; 1] = ["onPreBuild.bat"];
    /// The names of scripts to run after building.
    pub(super) static POST_BUILD_SCRIPT_NAMES: [&str; 1] = ["onPostBuild.bat"];
}

/// Defines constant values, specific to the OS.
#[cfg(not(target_os = "windows"))]
mod constants {
    /// The names of scripts to run before building.
    pub(super) static PRE_BUILD_SCRIPT_NAMES: [&str; 2] = ["onPreBuild.sh", "onPreBuild"];
    /// The names of scripts to run after building.
    pub(super) static POST_BUILD_SCRIPT_NAMES: [&str; 2] = ["onPostBuild.sh", "onPostBuild"];
}

/// Manages the execution of build scripts.
#[derive(Debug, Default)]
pub struct Scripts {
    /// The paths of scripts to run before building.
    on_pre_build: Vec<PathBuf>,
    /// The paths of scripts to run after building.
    on_post_build: Vec<PathBuf>,
}

/// Represents an error that can occur in relation to build scripts.
pub enum ScriptError {
    /// The script could not be executed, for example due to a missing file.
    CouldNotExecute(String),
    /// The script was executed, but returned a non-zero exit code.
    ScriptFailure {
        /// The name of the script.
        script: String,
        /// The exit code returned by the script.
        exit_code: i32,
        /// The output of the script.
        stdout: String,
        /// The error output of the script.
        stderr: String,
    },
}

impl Scripts {
    /// Loads scripts from the given directory.
    pub fn from_directory(dir: impl AsRef<Path>) -> Self {
        if let Ok(dir) = read_dir(dir.as_ref()) {
            let mut scripts = Self::default();

            for entry in dir.flatten() {
                if entry.path().is_file() {
                    if constants::PRE_BUILD_SCRIPT_NAMES
                        .contains(&entry.file_name().to_string_lossy().as_ref())
                    {
                        scripts.on_pre_build.push(entry.path());
                    } else if constants::POST_BUILD_SCRIPT_NAMES
                        .contains(&entry.file_name().to_string_lossy().as_ref())
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

    /// Executes pre-build scripts.
    pub fn execute_pre_build(&self) -> Result<(), ScriptError> {
        self.execute(&self.on_pre_build)
    }

    /// Executes post-build scripts.
    pub fn execute_post_build(&self) -> Result<(), ScriptError> {
        self.execute(&self.on_post_build)
    }

    /// Executes the given scripts.
    fn execute(&self, scripts: &[PathBuf]) -> Result<(), ScriptError> {
        for script in scripts {
            log!(
                "Executing",
                "script `{}`",
                script.file_name().unwrap().to_string_lossy()
            );

            #[cfg(target_os = "windows")]
            let output = Command::new(script).output().map_err(|_| {
                ScriptError::CouldNotExecute(
                    script.file_name().unwrap().to_string_lossy().to_string(),
                )
            })?;

            #[cfg(not(target_os = "windows"))]
            let output = Command::new("sh").arg(script).output().map_err(|_| {
                ScriptError::CouldNotExecute(
                    script.file_name().unwrap().to_string_lossy().to_string(),
                )
            })?;

            if !output.status.success() {
                return Err(ScriptError::ScriptFailure {
                    script: script.file_name().unwrap().to_string_lossy().to_string(),
                    exit_code: output.status.code().unwrap_or(-1),
                    stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
                    stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
                });
            }
        }

        Ok(())
    }
}
