//! Provides the plugin system for Stuart.

use crate::functions::FunctionParser;
use crate::process::ProcessOutput;
use crate::{Environment, Stuart};

use humphrey_json::prelude::*;
use humphrey_json::Value;

use std::path::Path;

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
    /// The node parsers provided by the plugin.
    pub parsers: Vec<Box<dyn NodeParser>>,
}

/// Represents a type that can parse a raw filesystem node.
pub trait NodeParser {
    /// Returns the file extensions that this parser can parse.
    fn extensions(&self) -> Vec<&'static str>;

    /// Parses the node, returning the parsed contents within a type that implements `NodeProcessor` so they can then be processed.
    fn parse(&self, contents: &[u8], path: &Path) -> Result<Box<dyn NodeProcessor>, String>;
}

/// Represents a type that contains the parsed contents of a node, which can be processed.
pub trait NodeProcessor {
    /// Processes the parsed contents in the given environment, retuning the processed output.
    fn process(&self, processor: &Stuart, env: Environment) -> Result<ProcessOutput, String>;

    /// Returns a JSON representation of the parsed contents to appear within the metadata of the build.
    fn to_json(&self) -> Value {
        json!({ "type": "custom" })
    }
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
///     parsers: [
///         SomeNodeParser
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
        parsers: [
            $($parser:expr),*
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
                parsers: vec![
                    $(
                        Box::new($parser)
                    ),*
                ],
            };

            Box::into_raw(Box::new(plugin))
        }
    };
}
