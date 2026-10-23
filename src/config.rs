use std::path::PathBuf;

#[derive(Debug)]
pub struct Config {
    pub urls: Vec<String>,
    pub input_file: Option<PathBuf>,
    pub output_path: PathBuf,
    pub max: u64,
    pub tries: u64,
    pub reverse_order: bool,
    pub proxy_file: Option<PathBuf>,
    pub proxy_downloads: bool,
    pub flare_solver: Option<String>,
}

impl Config {
    pub fn new(
        urls: Vec<String>,
        input_file: Option<PathBuf>,
        output_path: PathBuf,
        max: u64,
        tries: u64,
        reverse_order: bool,
        proxy_file: Option<PathBuf>,
        proxy_downloads: bool,
        flare_solver: Option<String>,
    ) -> Self {
        Self {
            urls,
            input_file,
            output_path,
            max,
            tries,
            reverse_order,
            proxy_file,
            proxy_downloads,
            flare_solver,
        }
    }
}
