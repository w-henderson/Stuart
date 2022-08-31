//! Provides methods to parse Stuart configuration files into the core [`Config`] type.

pub mod git;

use stuart_core::Config;

use serde_derive::Deserialize;

/// Raw, unparsed configuration information from the TOML file.
#[derive(Deserialize)]
struct RawConfig {
    /// Site configuration.
    site: Site,
    /// Settings configuration.
    settings: Option<Settings>,
}

/// Raw, unparsed site configuration information from the TOML file.
#[derive(Deserialize)]
struct Site {
    /// The name of the site.
    name: String,
    /// The author of the site.
    author: Option<String>,
}

/// Raw, unparsed settings configuration information from the TOML file.
#[derive(Deserialize)]
struct Settings {
    /// Whether to remove HTML extensions by creating folders containing `index.html` files.
    strip_extensions: Option<bool>,
    /// Whether to save JSON files.
    save_data_files: Option<bool>,
    /// Whether to output the build metadata.
    save_metadata: Option<bool>,
}

/// Attempts to load the configuration from the given TOML file.
pub fn load(string: &str) -> Result<Config, toml::de::Error> {
    let raw_config: RawConfig = toml::from_str(string)?;
    Ok(raw_config.into())
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
