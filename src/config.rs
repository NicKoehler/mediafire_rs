use std::path::PathBuf;

#[derive(Debug)]
pub struct Config {
    pub urls: Vec<String>,
    pub output_path: PathBuf,
    pub max: u64,
    pub tries: u64,
    pub reverse_order: bool,
    pub proxy_file: Option<PathBuf>,
    pub proxy_downloads: bool,
}

impl Config {
    pub fn new(
        urls: Vec<String>,
        output_path: PathBuf,
        max: u64,
        tries: u64,
        reverse_order: bool,
        proxy_file: Option<PathBuf>,
        proxy_downloads: bool,
    ) -> Self {
        Self {
            urls,
            output_path,
            max,
            tries,
            reverse_order,
            proxy_file,
            proxy_downloads,
        }
    }
}
