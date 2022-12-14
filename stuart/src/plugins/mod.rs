//! Provides support for dynamically-loaded plugins.

mod source;

use crate::config::git;
use crate::error::StuartError;

use stuart_core::error::{Error, FsError};
use stuart_core::plugins::{Manager, Plugin};

use libloading::Library;

use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Represents an external function that initializes a plugin.
type PluginInitFn = unsafe extern "C" fn() -> *mut Plugin;

/// A plugin manager that deals with dynamically-loaded plugins.
#[derive(Default)]
pub struct DynamicPluginManager {
    /// The plugins loaded by the plugin manager.
    plugins: Vec<Plugin>,
    /// The libraries which belong to the loaded plugins.
    libraries: Vec<Library>,
}

/// Attempts to load the plugins configured in the hash map.
///
/// This function will automatically detect the source kind and load it appropriately.
///
/// Example configuration:
/// ```toml
/// [dependencies]
/// plugin = "/path/to/plugin.so"
/// another_plugin = "https://github.com/username/another_plugin.git"
/// yet_another_plugin = "/path/to/cargo_project"
/// ```
pub fn load(
    plugins: &Option<HashMap<String, String>>,
    root: &Path,
) -> Result<DynamicPluginManager, Box<dyn StuartError>> {
    let plugins_start = Instant::now();

    let mut manager = DynamicPluginManager::new();

    if let Some(plugins) = plugins {
        for (name, src) in plugins {
            if let Ok(source) = PathBuf::try_from(src) {
                if source.exists() && source.is_file() {
                    log!("Loading", "plugin `{}`", name);

                    unsafe { manager.load(source)? };

                    continue;
                } else if source.join("Cargo.toml").exists() {
                    log!("Compiling", "plugin `{}`", name);

                    let path = source::build_cargo_project(&source)?;

                    unsafe { manager.load(path)? };

                    continue;
                }
            }

            if git::exists(src) {
                let repo_dir = root.join(format!("_build/plugins/{}", name));
                let repo_dir_string = repo_dir
                    .to_string_lossy()
                    .to_string()
                    .trim_start_matches("\\\\?\\")
                    .to_string();

                if !repo_dir.exists() {
                    log!("Cloning", "plugin `{}` from `{}`", name, src);

                    create_dir_all(root.join("_build/plugins"))
                        .map_err(|_| Error::Fs(FsError::Write))?;

                    if !git::clone(src, &repo_dir_string) {
                        Err(format!(
                            "failed to clone Git repository for plugin `{}`",
                            name
                        ))?;
                    }
                } else {
                    log!("Pulling", "plugin `{}` from `{}`", name, src);

                    if !git::pull(&repo_dir_string) {
                        Err(format!(
                            "failed to pull Git repository for plugin `{}`",
                            name
                        ))?;
                    }
                }

                let project = source::find_cargo_project(&repo_dir, name)
                    .ok_or_else(|| format!("failed to find plugin `{}` in Git repository", name))?;

                log!("Compiling", "plugin `{}`", name);

                let path = source::build_cargo_project(&project)?;

                unsafe { manager.load(path)? };

                continue;
            }

            Err(format!("invalid source for plugin `{}`", name))?;
        }
    }

    let plugins_duration = (plugins_start.elapsed().as_micros() / 100) as f64 / 10.0;

    if !manager.plugins().is_empty() {
        log!(
            "Loaded",
            "{} plugin(s) in {}ms\n",
            manager.plugins().len(),
            plugins_duration
        );
    }

    Ok(manager)
}

impl DynamicPluginManager {
    /// Creates a new, empty plugin manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Attempts to load a plugin from the given path.
    ///
    /// # Safety
    ///
    /// Calls foreign code. The safety of this function is dependent on the safety of the foreign code.
    pub unsafe fn load(&mut self, path: impl AsRef<Path>) -> Result<(), String> {
        let library = Library::new(path.as_ref()).map_err(|e| e.to_string())?;
        self.libraries.push(library);

        let library = self.libraries.last().unwrap();
        let init_fn = library
            .get::<PluginInitFn>(b"_stuart_plugin_init")
            .map_err(|e| e.to_string())?;

        let raw_plugin = init_fn();
        let plugin = Box::from_raw(raw_plugin);
        self.plugins.push(*plugin);

        Ok(())
    }
}

impl Manager for DynamicPluginManager {
    fn plugins(&self) -> &[Plugin] {
        &self.plugins
    }
}
