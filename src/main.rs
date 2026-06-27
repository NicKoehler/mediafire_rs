use crate::cli::parse_args;
use crate::io::load_proxies;
use crate::io::load_urls;
use crate::progress::{build_total_progress_bar, prepare_total_bar_for_download};
use crate::report::print_failures;
use crate::styles::ProgressStyles;
use crate::workers::spawn_workers;

use colored::*;
use futures::future::join_all;
use md_downloader::MediafireDownloader;
use md_downloader::types::{DownloadError, DownloadJob};
use std::sync::Arc;
use tokio::sync::Mutex;

mod cli;
mod config;
mod download;
mod io;
mod progress;
mod report;
mod styles;
mod workers;

#[tokio::main]
async fn main() -> Result<(), DownloadError> {
    let config = parse_args();
    let mut urls = config.urls.clone();
    if let Some(file) = config.input_file.clone() {
        match load_urls(&file) {
            Ok(data) => {
                urls.extend(data);
            }
            Err(e) => {
                eprintln!("Failed to load {}: {}", file.display(), e);
            }
        }
    }
    let proxies = load_proxies(config.proxy_file.clone());

    let downloader = MediafireDownloader::new(config.tries)?
        .reverse_downloads(config.reverse_order)
        .set_proxies(proxies, config.proxy_downloads)?;

    let styles = Arc::new(ProgressStyles::new());
    let (multi, total_bar) = build_total_progress_bar();

    let jobs = downloader
        .get_download_jobs_with_progress(&urls, config.output_path, |filename| {
            total_bar.set_message(filename.to_string())
        })
        .await?;

    if jobs.is_empty() {
        println!("{}", "Warning: No files to download".yellow());
        return Ok(());
    }

    prepare_total_bar_for_download(&total_bar, jobs.len());

    let jobs = Arc::new(Mutex::new(jobs));
    let successful = Arc::new(Mutex::new(Vec::<DownloadJob>::new()));
    let failed = Arc::new(Mutex::new(Vec::<(DownloadJob, DownloadError)>::new()));

    let handles = spawn_workers(
        config.max,
        config.tries,
        jobs,
        multi.clone(),
        total_bar.clone(),
        styles,
        successful.clone(),
        failed.clone(),
    );

    join_all(handles).await;

    total_bar.finish();

    print_failures(failed).await;

    Ok(())
}
