//! Provides methods to parse Stuart configuration files into the core [`Config`] type.

pub mod git;

use stuart_core::Config;

use serde_derive::Deserialize;

use std::collections::HashMap;

/// Raw, unparsed configuration information from the TOML file.
#[derive(Clone, Deserialize)]
pub struct RawConfig {
    /// Site configuration.
    pub site: Site,
    /// Settings configuration.
    pub settings: Option<Settings>,
    /// Dependencies.
    pub dependencies: Option<HashMap<String, String>>,
}

/// Raw, unparsed site configuration information from the TOML file.
#[derive(Clone, Deserialize)]
pub struct Site {
    /// The name of the site.
    pub name: String,
    /// The author of the site.
    pub author: Option<String>,
}

/// Raw, unparsed settings configuration information from the TOML file.
#[derive(Clone, Deserialize)]
pub struct Settings {
    /// Whether to remove HTML extensions by creating folders containing `index.html` files.
    pub strip_extensions: Option<bool>,
    /// Whether to save JSON files.
    pub save_data_files: Option<bool>,
    /// Whether to output the build metadata.
    pub save_metadata: Option<bool>,
}

/// Attempts to load the configuration from the given TOML file.
pub fn load(string: &str) -> Result<RawConfig, toml::de::Error> {
    toml::from_str(string)
}

impl From<RawConfig> for Config {
    fn from(raw: RawConfig) -> Self {
        let default = Config::default();

        Config {
            name: raw.site.name,
            author: raw.site.author,
            strip_extensions: raw
                .settings
                .as_ref()
                .and_then(|settings| settings.strip_extensions)
                .unwrap_or(default.strip_extensions),
            save_data_files: raw
                .settings
                .as_ref()
                .and_then(|settings| settings.save_data_files)
                .unwrap_or(default.save_data_files),
            save_metadata: raw
                .settings
                .as_ref()
                .and_then(|settings| settings.save_metadata)
                .unwrap_or(default.save_metadata),
        }
    }
}
