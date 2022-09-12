//! Provides the plugin system for Stuart.

use crate::functions::FunctionParser;

/// Represents a type that can manage plugins.
///
/// Stuart can be passed a plugin manager using the `with_plugins` method. It will then use this trait
///   to query plugins for their parsers and functions.
///
/// This trait is necessary to allow Stuart a single interface for plugins, whether they are statically linked
///   or dynamically loaded. It is automatically implemented for basic collections of plugins.
pub trait Manager {
    /// Returns the plugins loaded by the plugin manager.
    fn plugins(&self) -> &[Plugin];
}

/// Represents a plugin.
pub struct Plugin {
    /// The name of the plugin.
    ///
    /// This will prefix all functions provided by the plugin, for example `plugin_name::function_name`.
    pub name: String,
    /// The version of the plugin.
    pub version: String,
    /// The functions provided by the plugin.
    pub functions: Vec<Box<dyn FunctionParser>>,
}

impl<T> Manager for T
where
    T: AsRef<[Plugin]>,
{
    fn plugins(&self) -> &[Plugin] {
        self.as_ref()
    }
}

/// Declares a dynamically-loadable plugin.
///
/// ## Example
/// ```
/// declare_plugin! {
///     name: "my_plugin",
///     version: "1.0.0",
///     functions: [
///         SomeFunctionParser,
///         AnotherFunctionParser
///     ],
/// }
/// ```
#[macro_export]
macro_rules! declare_plugin {
    (
        name: $name:expr,
        version: $version:expr,
        functions: [
            $($function:expr),*
        ],
    ) => {
        #[no_mangle]
        pub extern "C" fn _stuart_plugin_init() -> *mut ::stuart_core::plugins::Plugin {
            let plugin = ::stuart_core::plugins::Plugin {
                name: $name.into(),
                version: $version.into(),
                functions: vec![
                    $(
                        Box::new($function)
                    ),*
                ],
            };

            Box::into_raw(Box::new(plugin))
        }
    };
}
