//! Provides support for dynamically-loaded plugins.

use stuart_core::plugins::{Manager, Plugin};

use libloading::Library;

use std::collections::HashMap;
use std::path::Path;

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
pub fn load(plugins: &Option<HashMap<String, String>>) -> Result<DynamicPluginManager, String> {
    let mut manager = DynamicPluginManager::new();

    if let Some(plugins) = plugins {
        for path in plugins.values() {
            unsafe { manager.load(path)? };
        }
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
