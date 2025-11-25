use std::path::PathBuf;

#[derive(Debug,Clone)]
pub(crate) struct AppConfig {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) root_dirpath: PathBuf,
}

impl AppConfig {
    pub(crate) fn new() -> AppConfig {
        let cur_root_dir = std::env::current_dir().expect("Failed to get current directory");
        AppConfig {
            host: "127.0.0.1".to_string(),
            port: 3000,
            root_dirpath: cur_root_dir,
        }
    }
}
