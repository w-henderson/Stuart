//! Provides support for dynamically-loaded plugins.

mod source;

#[cfg(feature = "js")]
mod js;

use crate::config::git;
use crate::error::StuartError;

use stuart_core::error::{Error, FsError};
use stuart_core::plugins::{Manager, Plugin};

use libloading::Library;

use std::collections::HashMap;
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
/// Sources can be separated within the string with a semicolon (`;`), and will
/// be tried in order until one succeeds or all fail.
///
/// Example configuration:
/// ```toml
/// [dependencies]
/// plugin = "/path/to/plugin.so"
/// git_plugin = "https://github.com/username/another_plugin.git"
/// src_plugin = "/path/to/cargo_project"
/// download_plugin = "https://example.com/plugin.so"
/// os_independent_plugin = "/path/to/plugin.dll;/path/to/plugin.so"
/// ```
pub fn load(
    plugins: &Option<HashMap<String, String>>,
    root: &Path,
) -> Result<DynamicPluginManager, Box<dyn StuartError>> {
    let plugins_start = Instant::now();

    let mut manager = DynamicPluginManager::new();

    if let Some(plugins) = plugins {
        'outer: for (name, src) in plugins {
            let mut e: Option<Box<dyn StuartError>> = None;

            for source in src.split(';') {
                #[cfg(target_os = "windows")]
                if source.ends_with(".so") {
                    log!(
                        "Skipping",
                        "plugin file `{}` (not supported on Windows)",
                        source
                    );
                    continue;
                }

                #[cfg(not(target_os = "windows"))]
                if source.ends_with(".dll") {
                    log!(
                        "Skipping",
                        "plugin file `{}` (not supported on non-Windows platforms)",
                        source
                    );
                    continue;
                }

                #[cfg(not(feature = "js"))]
                if source.ends_with(".js") || source.ends_with(".mjs") {
                    log!(
                        "Skipping",
                        "plugin file `{}` (JavaScript support is not enabled)",
                        source
                    );
                    continue;
                }

                if let Err(err) = load_from_source(&mut manager, name, source, root) {
                    if e.is_none() {
                        err.print();
                        e = Some(err);
                    }
                } else {
                    continue 'outer;
                }
            }

            return Err(Box::new("Plugins failed to load".to_string()));
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

/// Attempts to load one specific plugin from the given source.
fn load_from_source(
    manager: &mut DynamicPluginManager,
    name: &str,
    src: &str,
    root: &Path,
) -> Result<(), Box<dyn StuartError>> {
    let source = root.join(src);

    if source.exists() && source.is_file() {
        log!("Loading", "plugin `{}` from `{}`", name, src);

        manager.load(source)?;

        Ok(())
    } else if source.join("Cargo.toml").exists() {
        log!("Compiling", "plugin `{}` from `{}`", name, src);

        let path = source::build_cargo_project(&source)?;

        unsafe { manager.load_binary(path)? };

        Ok(())
    } else if git::exists(src) {
        let repo_dir = root.join(format!("_build/plugins/{}", name));
        let repo_dir_string = repo_dir
            .to_string_lossy()
            .to_string()
            .trim_start_matches("\\\\?\\")
            .to_string();

        if !repo_dir.exists() {
            log!("Cloning", "plugin `{}` from `{}`", name, src);

            create_dir_all(root.join("_build/plugins")).map_err(|_| Error::Fs(FsError::Write))?;

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

        let path = source::build_cargo_project(project)?;

        unsafe { manager.load_binary(path)? };

        Ok(())
    } else if let Some(plugin) = source::download_plugin(src) {
        log!("Downloading", "plugin `{}` from `{}`", name, src);

        let plugin_dir = root.join(format!("_build/plugins/{}", name));
        let plugin_path = plugin_dir.join(src.rsplit('/').next().unwrap());

        if !plugin_dir.exists() {
            create_dir_all(&plugin_dir).map_err(|_| Error::Fs(FsError::Write))?;
        }

        std::fs::write(&plugin_path, plugin).map_err(|_| Error::Fs(FsError::Write))?;

        manager.load(plugin_path)?;

        Ok(())
    } else {
        Err(format!("invalid source for plugin `{}`", name))?
    }
}

impl DynamicPluginManager {
    /// Creates a new, empty plugin manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Attempts to load a plugin from the given path.
    pub fn load(&mut self, path: impl AsRef<Path>) -> Result<(), String> {
        let path = path.as_ref();
        let ext = path.extension().unwrap_or_default().to_string_lossy();

        if ext == "js" || ext == "mjs" {
            #[cfg(feature = "js")]
            self.load_js(path)
        } else {
            unsafe { self.load_binary(path) }
        }
    }

    /// Attempts to load a binary plugin from the given path.
    ///
    /// # Safety
    ///
    /// Calls foreign code. The safety of this function is dependent on the safety of the foreign code.
    pub unsafe fn load_binary(&mut self, path: impl AsRef<Path>) -> Result<(), String> {
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

    /// Attempts to load a JavaScript plugin from the given path.
    #[cfg(feature = "js")]
    pub fn load_js(&mut self, path: impl AsRef<Path>) -> Result<(), String> {
        let plugin = js::load_js_plugin(path)?;
        self.plugins.push(plugin);

        Ok(())
    }
}

impl Manager for DynamicPluginManager {
    fn plugins(&self) -> &[Plugin] {
        &self.plugins
    }
}
