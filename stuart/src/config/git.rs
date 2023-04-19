//! Provides functionality for interfacing with Git.
//!
//! This is used to get user information for the `author` field, as well as initialising new Git repositories.

use std::process::Command;

/// Gets the user's name from Git.
pub fn get_user_name() -> Option<String> {
    let output = Command::new("git")
        .args(["config", "--get", "user.name"])
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// Gets the user's email from Git.
pub fn get_user_email() -> Option<String> {
    let output = Command::new("git")
        .args(["config", "--get", "user.email"])
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// Initialises a new Git repository in the given directory.
pub fn init_repository(path: &str) -> bool {
    Command::new("git")
        .arg("init")
        .arg(path)
        .output()
        .ok()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Checks whether a remote repository exists at the given URL.
pub fn exists(url: &str) -> bool {
    Command::new("git")
        .args(["ls-remote", url])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Clones the repository at the given URL into the given directory.
///
/// Returns `true` if the clone was successful, `false` otherwise.
pub fn clone(url: &str, path: &str) -> bool {
    Command::new("git")
        .args(["clone", url, path, "--depth", "1"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Attempts to pull the latest changes from the remote repository into the given directory.
///
/// Returns `true` if the pull was successful, `false` otherwise.
pub fn pull(path: &str) -> bool {
    Command::new("git")
        .args(["-C", path, "pull"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
