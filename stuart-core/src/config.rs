#[derive(Clone, Debug)]
pub struct Config {
    pub name: String,
    pub author: Option<String>,
    pub strip_extensions: bool,
    pub save_data_files: bool,
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
