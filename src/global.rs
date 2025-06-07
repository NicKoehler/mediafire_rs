use deadqueue::unlimited::Queue;
use indicatif::{ProgressBar, ProgressStyle};
use lazy_static::lazy_static;
use tokio::sync::Mutex;

use crate::types::download::DownloadJob;

lazy_static! {
    pub static ref PROGRESS_STYLE_TOTAL_START: ProgressStyle = ProgressStyle::default_bar()
        .progress_chars("-> ")
        .template("{spinner} Fetching data · {msg:.blue}")
        .unwrap();

    pub static ref PROGRESS_STYLE_TOTAL_DOWNLOAD: ProgressStyle = ProgressStyle::default_bar()
        .progress_chars("-> ")
        .template("[{bar:30.blue}] {pos}/{len} ({percent}%) · {msg:.green} · {prefix:.red}")
        .unwrap();

    pub static ref PROGRESS_STYLE_ERROR: ProgressStyle = ProgressStyle::default_bar()
        .progress_chars("-> ")
        .template("[{bar:30.green}] · {msg} · {prefix:.red}")
        .unwrap();

    pub static ref PROGRESS_STYLE: ProgressStyle = ProgressStyle::default_bar()
        .progress_chars("-> ")
        .template("[{bar:30.green}] · {msg} · {prefix:.blue}")
        .unwrap();

    pub static ref PROGRESS_STYLE_DOWNLOAD: ProgressStyle = ProgressStyle::default_bar()
        .progress_chars("-> ")
        .template("[{bar:30.green}] · {msg} · {percent}% ({bytes:.magenta}/{total_bytes:.magenta}) · {prefix:.blue}")
        .unwrap();

    pub static ref QUEUE: Queue<DownloadJob> = Queue::new();

    pub static ref MULTI_PROGRESS_BAR: indicatif::MultiProgress = indicatif::MultiProgress::new();

    pub static ref TOTAL_PROGRESS_BAR: ProgressBar = MULTI_PROGRESS_BAR.add(ProgressBar::new(0));

    pub static ref SUCCESSFUL_DOWNLOADS: Mutex<Vec<DownloadJob>> = Mutex::new(Vec::new());

    pub static ref FAILED_DOWNLOADS: Mutex<Vec<DownloadJob>> = Mutex::new(Vec::new());
}
