use crate::config::Config;
use crate::download::download_file;
use crate::styles::ProgressStyles;

use clap::{ArgAction, arg, command, value_parser};
use colored::*;
use futures::future::join_all;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use md_downloader::MediafireDownloader;
use md_downloader::types::{DownloadError, DownloadJob};
use std::collections::BinaryHeap;
use std::{
    fs::File,
    io::{self, BufRead},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};
use tokio::sync::Mutex;

mod config;
mod download;
mod styles;

#[tokio::main]
async fn main() -> Result<(), DownloadError> {
    let config = parse_args();
    let proxies = load_proxies(config.proxy_file.clone());

    let downloader = MediafireDownloader::new(config.tries)?
        .reverse_downloads(config.reverse_order)
        .set_proxies(proxies, config.proxy_downloads)?;

    let styles = Arc::new(ProgressStyles::new());
    let (multi, total_bar) = build_total_progress_bar();

    let jobs = downloader
        .get_download_jobs_with_progress(&config.urls, config.output_path, |filename| {
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

fn parse_args() -> Config {
    let matches = command!()
        .author("NicKoehler")
        .color(clap::ColorChoice::Always)
        .arg(
            arg!([URLS] "List of folders or files to download")
                .required(true)
                .value_parser(value_parser!(String))
                .num_args(1..),
        )
        .arg(
            arg!(-o --output <OUTPUT> "Output directory")
                .default_value(".")
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            arg!(-m --max <MAX> "Maximum concurrent downloads")
                .default_value("10")
                .value_parser(value_parser!(u64).range(1..=100)),
        )
        .arg(
            arg!(-t --tries <TRIES> "Maximum retries per download")
                .default_value("1")
                .value_parser(value_parser!(u64).range(1..=10)),
        )
        .arg(arg!(-r --reverse "Download largest files first").action(ArgAction::SetTrue))
        .arg(
            arg!(-p --proxy <FILE> "File containing proxy list")
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(arg!(--"proxy-download" "Use proxies for file downloads").action(ArgAction::SetTrue))
        .get_matches();

    Config::new(
        matches.get_many("URLS").unwrap().cloned().collect(),
        matches.get_one::<PathBuf>("output").unwrap().clone(),
        *matches.get_one::<u64>("max").unwrap(),
        *matches.get_one::<u64>("tries").unwrap(),
        *matches.get_one::<bool>("reverse").unwrap(),
        matches.get_one::<PathBuf>("proxy").cloned(),
        *matches.get_one::<bool>("proxy-download").unwrap(),
    )
}

fn load_proxies(path: Option<PathBuf>) -> Option<Vec<String>> {
    path.map(|p| {
        File::open(p)
            .map(io::BufReader::new)
            .map(|reader| reader.lines().map_while(Result::ok).collect::<Vec<_>>())
            .unwrap_or_default()
    })
}

fn build_total_progress_bar() -> (Arc<MultiProgress>, Arc<ProgressBar>) {
    let multi = Arc::new(MultiProgress::new());
    let total_bar = Arc::new(multi.add(ProgressBar::new(0)));

    total_bar.enable_steady_tick(Duration::from_millis(120));
    total_bar.set_style(
        ProgressStyle::default_bar()
            .progress_chars("-> ")
            .template("{spinner} Fetching data · {msg:.blue}")
            .unwrap(),
    );

    (multi, total_bar)
}

fn prepare_total_bar_for_download(bar: &ProgressBar, job_count: usize) {
    bar.disable_steady_tick();
    bar.set_length(job_count as u64);
    bar.set_style(
        ProgressStyle::default_bar()
            .progress_chars("-> ")
            .template("[{bar:30.blue}] {pos}/{len} ({percent}%) · {msg:.green} · {prefix:.red}")
            .unwrap(),
    );
    bar.set_message("Downloading");
}

fn spawn_workers(
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

async fn print_failures(failed: Arc<Mutex<Vec<(DownloadJob, DownloadError)>>>) {
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
