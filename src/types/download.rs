use crate::types::file::File;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DownloadJob {
    pub file: File,
    pub path: PathBuf,
}

impl DownloadJob {
    pub fn new(file: File, path: PathBuf) -> Self {
        Self { file, path }
    }
}
