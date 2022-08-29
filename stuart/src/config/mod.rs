pub mod git;

use stuart_core::Config;

use serde_derive::Deserialize;

#[derive(Deserialize)]
struct RawConfig {
    site: Site,
    settings: Option<Settings>,
}

#[derive(Deserialize)]
struct Site {
    name: String,
    author: Option<String>,
}

#[derive(Deserialize)]
struct Settings {
    strip_extensions: Option<bool>,
    save_data_files: Option<bool>,
}

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
        }
    }
}
