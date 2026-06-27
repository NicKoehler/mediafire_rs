use indicatif::{MultiProgress, ProgressBar};
use md_downloader::types::{DownloadError, DownloadJob, DownloadProgress};
use std::sync::Arc;
use tokio::time::{Duration, sleep};

use crate::ProgressStyles;

const RETRY_SECS: u64 = 2;

pub async fn download_file(
    download_job: &DownloadJob,
    mut tries: u64,
    multi: Arc<MultiProgress>,
    styles: Arc<ProgressStyles>,
) -> Result<(), DownloadError> {
    let bar = multi.insert_from_back(1, ProgressBar::new(0));
    let filename = download_job.filename.clone();
    let mut last_error: Option<DownloadError> = None;

    while tries > 0 {
        bar.reset();
        bar.set_style(styles.normal.clone());
        bar.set_prefix(filename.clone());

        if let Err(e) = attempt_download(download_job, &bar, &styles).await {
            tries -= 1;
            last_error = Some(e);

            bar.set_style(styles.error.clone());
            bar.set_message("🔃");

            if tries > 0 {
                bar.set_prefix(format!("Retrying... {} left", tries));
            } else {
                bar.set_prefix("No more retries left");
            }

            sleep(Duration::from_secs(RETRY_SECS)).await;
        } else {
            return Ok(());
        }
    }

    bar.set_prefix(filename);
    bar.set_style(styles.error.clone());
    bar.abandon_with_message("❌");

    Err(last_error.unwrap())
}

async fn attempt_download(
    download_job: &DownloadJob,
    bar: &ProgressBar,
    styles: &ProgressStyles,
) -> Result<(), DownloadError> {
    let filename = download_job.filename.clone();

    download_job
        .download_with_progress(|status| {
            match status {
                DownloadProgress::Downloading(partial, total) => {
                    bar.set_style(styles.download.clone());
                    bar.set_prefix(filename.clone());
                    bar.set_message("🔽");
                    bar.set_length(total);
                    bar.set_position(partial);
                }
                DownloadProgress::TryResuming => {
                    bar.set_style(styles.normal.clone());
                    bar.set_message("💫");
                    bar.set_prefix("Try resuming...");
                    bar.tick();
                }
                DownloadProgress::GettingLink => {
                    bar.set_style(styles.normal.clone());
                    bar.set_message("🌀");
                    bar.set_prefix("Getting link...");
                    bar.tick();
                }
                DownloadProgress::CheckingHash => {
                    bar.set_style(styles.normal.clone());
                    bar.set_message("💾");
                    bar.set_prefix("Checking hash...");
                    bar.tick();
                }
                DownloadProgress::Done => {
                    bar.set_prefix(filename.clone());
                    bar.set_style(styles.normal.clone());
                    bar.abandon_with_message("✅");
                    bar.tick();
                }
            };
        })
        .await
}
