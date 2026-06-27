use crate::download::download_file;
use crate::styles::ProgressStyles;

use indicatif::{MultiProgress, ProgressBar};
use md_downloader::types::{DownloadError, DownloadJob};
use std::collections::BinaryHeap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn spawn_workers(
    max: u64,
    tries: u64,
    jobs: Arc<Mutex<BinaryHeap<DownloadJob>>>,
    multi: Arc<MultiProgress>,
    total_bar: Arc<ProgressBar>,
    styles: Arc<ProgressStyles>,
    successful: Arc<Mutex<Vec<DownloadJob>>>,
    failed: Arc<Mutex<Vec<(DownloadJob, DownloadError)>>>,
) -> Vec<tokio::task::JoinHandle<()>> {
    (0..max)
        .map(|_| {
            let jobs = jobs.clone();
            let multi = multi.clone();
            let total_bar = total_bar.clone();
            let styles = styles.clone();
            let successful = successful.clone();
            let failed = failed.clone();

            tokio::spawn(async move {
                loop {
                    let job = {
                        let mut queue = jobs.lock().await;
                        queue.pop()
                    };

                    let Some(job) = job else { break };

                    let result = download_file(&job, tries, multi.clone(), styles.clone()).await;

                    match result {
                        Ok(_) => successful.lock().await.push(job),
                        Err(e) => failed.lock().await.push((job, e)),
                    }

                    let failed_count = failed.lock().await.len();
                    let success_count = successful.lock().await.len();

                    total_bar.set_prefix(format!("Failed {}", failed_count));
                    total_bar.set_message(format!("Successful {}", success_count));
                    total_bar.inc(1);
                }
            })
        })
        .collect()
}
