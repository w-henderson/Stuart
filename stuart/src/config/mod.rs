#[derive(Debug)]
pub struct Config {
    pub strip_extensions: bool,
    pub save_data_files: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            strip_extensions: true,
            save_data_files: false,
        }
    }
}
