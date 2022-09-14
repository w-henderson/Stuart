//! Provides functionality for finding plugins from a source string.

use crate::scripts::ScriptError;

use std::fs::{read_dir, read_to_string};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Attempts to find the named Cargo project within the given directory.
pub fn find_cargo_project(root: impl AsRef<Path>, name: &str) -> Option<PathBuf> {
    let root = root.as_ref();

    for entry in read_dir(root).ok()?.flatten() {
        let metadata = entry.metadata().ok()?;

        if metadata.is_file()
            && entry.file_name() == "Cargo.toml"
            && get_project_name(&entry.path()) == Some(name.to_string())
        {
            return Some(entry.path().parent()?.to_path_buf());
        } else if metadata.is_dir() {
            if let Some(path) = find_cargo_project(entry.path(), name) {
                return Some(path);
            }
        }
    }

    None
}

/// Attempts to build the Cargo project at the given path, returning the path to the compiled plugin.
///
/// **Note:** this function may not work correctly in the case of workspace projects.
pub fn build_cargo_project(root: impl AsRef<Path>) -> Result<PathBuf, ScriptError> {
    let manifest = root.as_ref().join("Cargo.toml");

    let output = Command::new("cargo")
        .args(&["build", "--release", "--manifest-path"])
        .arg(&manifest)
        .output()
        .map_err(|_| ScriptError::CouldNotExecute("<build script>".to_string()))?;

    if !output.status.success() {
        return Err(ScriptError::ScriptFailure {
            script: format!(
                "cargo build --release --manifest-path {}",
                manifest.display()
            ),
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }

    #[cfg(target_os = "windows")]
    let target_file = root
        .as_ref()
        .join("target/release")
        .join(format!("{}.dll", get_project_name(&manifest).unwrap()));
    #[cfg(not(target_os = "windows"))]
    let target_file = root
        .as_ref()
        .join("target/release")
        .join(format!("lib{}.so", get_project_name(&manifest).unwrap()));

    Ok(target_file)
}

/// Attempts to get the name of the Cargo project defined by the given manifest.
fn get_project_name(manifest: &Path) -> Option<String> {
    let manifest = read_to_string(manifest).ok()?;
    let toml: toml::Value = toml::from_str(&manifest).ok()?;

    toml.get("package")?
        .get("name")?
        .as_str()
        .map(|s| s.to_string())
}
