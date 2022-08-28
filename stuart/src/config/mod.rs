use serde_derive::Deserialize;

#[derive(Debug)]
pub struct Config {
    pub name: String,
    pub author: Option<String>,
    pub strip_extensions: bool,
    pub save_data_files: bool,
}

#[derive(Deserialize)]
struct RawConfig {
    site: Site,
    settings: Settings,
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

impl Config {
    pub fn load(string: &str) -> Result<Config, toml::de::Error> {
        let raw_config: RawConfig = toml::from_str(string)?;
        Ok(raw_config.into())
    }
}

impl From<RawConfig> for Config {
    fn from(raw: RawConfig) -> Self {
        let default = Config::default();

        Config {
            name: raw.site.name,
            author: raw.site.author,
            strip_extensions: raw
                .settings
                .strip_extensions
                .unwrap_or(default.strip_extensions),
            save_data_files: raw
                .settings
                .save_data_files
                .unwrap_or(default.save_data_files),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            author: None,
            strip_extensions: true,
            save_data_files: false,
        }
    }
}
