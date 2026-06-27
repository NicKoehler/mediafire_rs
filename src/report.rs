use colored::*;
use md_downloader::types::{DownloadError, DownloadJob};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn print_failures(failed: Arc<Mutex<Vec<(DownloadJob, DownloadError)>>>) {
    let failed = failed.lock().await;

    if failed.is_empty() {
        return;
    }

    println!("\nFailed downloads:");
    for (job, error) in failed.iter() {
        println!(
            "{}",
            format!("{} · {} · {}", job.filename, error, job.download_link).red()
        );
    }
}
