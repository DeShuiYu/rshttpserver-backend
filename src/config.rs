use std::path::PathBuf;
use clap::Parser;

#[derive(Debug,Clone)]
pub(crate) struct AppConfig {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) root_dirpath: PathBuf,
}


#[derive(Parser, Debug)]
#[command(version="0.1")]
struct AppArgs {
    /// port
    #[arg(long)]
    port:Option<u16>,

    #[arg(short='d', long)]
    root:Option<PathBuf>,
}

impl AppConfig {
    pub(crate) fn new() -> AppConfig {
        let app_args = AppArgs::parse();
        let cur_root_dir = std::env::current_dir().expect("Failed to get current directory");
        AppConfig {
            host: "0.0.0.0".to_string(),
            port: if let Some(port) = app_args.port {port}else { 3000 },
            root_dirpath: if let Some(root) = app_args.root {
                if root.is_relative() {
                    cur_root_dir.join(root)
                }else {
                    root
                }
            }else { cur_root_dir },
        }
    }
}
