use crate::types::file::File;
use std::path::PathBuf;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DownloadJob {
    pub file: File,
    pub path: PathBuf,
}

impl PartialOrd for DownloadJob {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.file.size.partial_cmp(&other.file.size)
    }
}

impl Ord for DownloadJob {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.file.size.cmp(&other.file.size)
    }
}

impl DownloadJob {
    pub fn new(file: File, path: PathBuf) -> Self {
        Self { file, path }
    }
}
