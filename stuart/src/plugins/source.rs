use std::fs::{read_dir, read_to_string};
use std::path::{Path, PathBuf};
use std::process::Command;

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

pub fn build_cargo_project(root: impl AsRef<Path>) -> Option<PathBuf> {
    let manifest = root.as_ref().join("Cargo.toml");

    let output = Command::new("cargo")
        .args(&["build", "--release", "--manifest-path"])
        .arg(&manifest)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    #[cfg(target_os = "windows")]
    let target_file = root
        .as_ref()
        .join("target/release")
        .join(format!("{}.dll", get_project_name(&manifest)?));
    #[cfg(not(target_os = "windows"))]
    let target_file = root
        .as_ref()
        .join("target/release")
        .join(format!("lib{}.so", get_project_name(&manifest)?));

    if target_file.exists() {
        Some(target_file)
    } else {
        None
    }
}

fn get_project_name(manifest: &Path) -> Option<String> {
    let manifest = read_to_string(manifest).ok()?;
    let toml: toml::Value = toml::from_str(&manifest).ok()?;

    toml.get("package")?
        .get("name")?
        .as_str()
        .map(|s| s.to_string())
}
