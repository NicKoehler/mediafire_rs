mod api;
mod consts;
mod download;
mod types;
mod utils;

use crate::api::file;
use crate::api::folder;
use crate::download::{download_file, download_folder};
use crate::utils::{create_directory_if_not_exists, match_mediafire_valid_url};
use anyhow::anyhow;
use anyhow::Result;
use clap::{arg, command, value_parser};
use deadqueue::unlimited::Queue;
use indicatif::MultiProgress;
use indicatif::ProgressBar;
use indicatif::ProgressFinish;
use indicatif::ProgressStyle;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;
use types::download::DownloadJob;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = command!()
        .author("NicKoehler")
        .color(clap::ColorChoice::Always)
        .arg(
            arg!([URL] "Folder or file to download")
                .required(true)
                .value_parser(value_parser!(String)),
        )
        .arg(
            arg!(-o --output <OUTPUT> "Output directory")
                .required(false)
                .default_value(".")
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            arg!(-m --max <MAX> "Maximum number of concurrent downloads")
                .required(false)
                .default_value("10")
                .value_parser(value_parser!(usize)),
        )
        .get_matches();

    let url = matches.get_one::<String>("URL").unwrap();
    let path = matches.get_one::<PathBuf>("output").unwrap().to_path_buf();
    let max = *matches.get_one::<usize>("max").unwrap();
    let option = match_mediafire_valid_url(url);

    let total_downloads = Arc::new(Mutex::new(0));
    let total_failed = Arc::new(Mutex::new(0));

    let queue: Arc<Queue<DownloadJob>> = Arc::new(Queue::new());

    let multi_progress_bar = Arc::new(MultiProgress::new());

    let total_bar = Arc::new(
        multi_progress_bar
            .add(ProgressBar::new(0))
            .with_finish(ProgressFinish::AndLeave),
    );
    total_bar.enable_steady_tick(Duration::from_millis(120));
    total_bar.set_style(
        ProgressStyle::default_bar()
            .template("Fetching data · {msg} {spinner}")
            .unwrap(),
    );

    if let Some((mode, key)) = option {
        if mode == "folder" {
            let response = folder::get_info(&key).await;
            if let Ok(response) = response {
                if let Some(folder) = response.folder_info {
                    download_folder(
                        &key,
                        path.join(PathBuf::from(folder.name)),
                        1,
                        queue.clone(),
                        total_bar.clone(),
                    )
                    .await?;
                }
            }
        } else {
            create_directory_if_not_exists(&path).await?;
            let response = file::get_info(&key).await;
            if let Ok(response) = response {
                if let Some(file_info) = response.file_info {
                    let path = path.join(PathBuf::from(&file_info.filename));
                    queue.clone().push(DownloadJob::new(file_info.into(), path));
                }
            }
        }
    }

    total_bar.finish_with_message("Done 🎉");

    if queue.len() == 0 {
        return Err(anyhow!("No files to download"));
    }

    let total_bar = Arc::new(
        multi_progress_bar
            .add(ProgressBar::new(queue.len() as u64))
            .with_finish(ProgressFinish::AndLeave),
    );

    total_bar.set_style(
        total_bar
            .style()
            .template("[{bar:30}] {pos}/{len} ({percent}%) - {msg}")
            .unwrap()
            .progress_chars("=>-"),
    );

    for _ in 0..max {
        let queue = queue.clone();
        let multi_progress_bar = multi_progress_bar.clone();
        let total_downloads = total_downloads.clone();
        let total_failed = total_failed.clone();
        let total_bar = total_bar.clone();
        tokio::spawn(async move {
            loop {
                let task = queue.pop().await;
                let (downloaded, failed) = match download_file(task, &multi_progress_bar).await {
                    Ok(_) => (1, 0),
                    Err(_) => (0, 1),
                };

                *total_downloads.lock().unwrap() += downloaded;
                *total_failed.lock().unwrap() += failed;

                total_bar.set_message(format!(
                    "Successful downloads {} - Failed downloads {}",
                    *total_downloads.lock().unwrap(),
                    *total_failed.lock().unwrap()
                ));
                total_bar.inc(1);
            }
        });
    }

    if let Some(total_bar_length) = total_bar.length() {
        while total_bar.position() < total_bar_length {
            sleep(Duration::from_millis(100)).await;
        }
    }

    total_bar.finish();
    Ok(())
}
