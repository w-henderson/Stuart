//! Provides the [`Config`] type.

/// Represents the configuration of a project.
#[derive(Clone, Debug)]
pub struct Config {
    /// The name of the project.
    pub name: String,
    /// The author of the project.
    pub author: Option<String>,
    /// Whether to remove HTML extensions by creating folders containing `index.html` files.
    pub strip_extensions: bool,
    /// Whether to save JSON files.
    pub save_data_files: bool,
    /// Whether to output the build metadata.
    pub save_metadata: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            author: None,
            strip_extensions: true,
            save_data_files: false,
            save_metadata: false,
        }
    }
}
