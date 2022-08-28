use std::process::Command;

pub fn get_user_name() -> Option<String> {
    let output = Command::new("git")
        .args(&["config", "--get", "user.name"])
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

pub fn get_user_email() -> Option<String> {
    let output = Command::new("git")
        .args(&["config", "--get", "user.email"])
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

pub fn init_repository(path: &str) -> bool {
    Command::new("git")
        .arg("init")
        .arg(path)
        .output()
        .ok()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
