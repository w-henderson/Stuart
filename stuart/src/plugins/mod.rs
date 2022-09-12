use stuart_core::plugins::{Manager, Plugin};

use libloading::Library;

use std::collections::HashMap;
use std::path::Path;

type PluginInitFn = unsafe extern "C" fn() -> *mut Plugin;

#[derive(Default)]
pub struct DynamicPluginManager {
    plugins: Vec<Plugin>,
    libraries: Vec<Library>,
}

pub fn load(plugins: &Option<HashMap<String, String>>) -> Result<DynamicPluginManager, String> {
    let mut manager = DynamicPluginManager::new();

    if let Some(plugins) = plugins {
        for (_name, path) in plugins {
            unsafe { manager.load(path)? };
        }
    }

    Ok(manager)
}

impl DynamicPluginManager {
    pub fn new() -> Self {
        Self::default()
    }

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
