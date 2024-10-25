mod api;
mod download;
mod global;
mod types;
mod utils;

use crate::api::file;
use crate::api::folder;
use crate::download::{download_file, download_folder};
use crate::utils::{create_directory_if_not_exists, match_mediafire_valid_url};
use anyhow::anyhow;
use anyhow::Result;
use clap::{arg, command, value_parser};
use global::*;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;
use types::download::DownloadJob;

#[tokio::main]
async fn main() -> Result<()> {
    let get_matches = command!()
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
    let matches = get_matches;

    let url = matches.get_one::<String>("URL").unwrap();
    let path = matches.get_one::<PathBuf>("output").unwrap().to_path_buf();
    let max = *matches.get_one::<usize>("max").unwrap();
    let option = match_mediafire_valid_url(url);

    TOTAL_PROGRESS_BAR.enable_steady_tick(Duration::from_millis(120));
    TOTAL_PROGRESS_BAR.set_style(PROGRESS_STYLE_TOTAL_START.clone());

    if option.is_none() {
        return Err(anyhow!("Invalid Mediafire URL"));
    }

    let (mode, key) = option.unwrap();

    match mode.as_str() {
        "folder" => {
            if let Some(folder) = folder::get_info(&key).await?.folder_info {
                download_folder(&key, path.join(PathBuf::from(folder.name)), 1).await?;
            } else {
                return Err(anyhow!("Invalid Mediafire folder URL"));
            }
        }
        "file" | "file_premium" => {
            create_directory_if_not_exists(&path).await?;
            let response = file::get_info(&key).await?;
            if let Some(file_info) = response.file_info {
                let path = path.join(PathBuf::from(&file_info.filename));
                QUEUE.push(DownloadJob::new(file_info.into(), path));
            } else {
                return Err(anyhow!("Invalid Mediafire file URL"));
            }
        }
        _ => return Err(anyhow!("Invalid Mediafire URL")),
    }

    if QUEUE.len() == 0 {
        return Err(anyhow!("No files to download"));
    }

    TOTAL_PROGRESS_BAR.disable_steady_tick();
    TOTAL_PROGRESS_BAR.set_length(QUEUE.len() as u64);
    TOTAL_PROGRESS_BAR.set_style(PROGRESS_STYLE_TOTAL_DOWNLOAD.clone());

    TOTAL_PROGRESS_BAR.set_message("Downloading");

    for _ in 0..max {
        tokio::spawn(async move {
            loop {
                let task = QUEUE.pop().await;
                match download_file(&task).await {
                    Ok(_) => SUCCESSFUL_DOWNLOADS.lock().await.push(task),
                    Err(_) => FAILED_DOWNLOADS.lock().await.push(task),
                };

                TOTAL_PROGRESS_BAR.set_prefix(format!(
                    "Failed downloads {}",
                    FAILED_DOWNLOADS.lock().await.len()
                ));

                TOTAL_PROGRESS_BAR.set_message(format!(
                    "Successful downloads {}",
                    SUCCESSFUL_DOWNLOADS.lock().await.len()
                ));

                TOTAL_PROGRESS_BAR.inc(1);
            }
        });
    }

    if let Some(total_bar_length) = TOTAL_PROGRESS_BAR.length() {
        while TOTAL_PROGRESS_BAR.position() < total_bar_length {
            sleep(Duration::from_millis(100)).await;
        }
    }

    TOTAL_PROGRESS_BAR.finish();

    let failed = FAILED_DOWNLOADS.lock().await;
    if failed.len() > 0 {
        println!("Failed downloads:");
        failed.iter().for_each(|job| {
            println!(
                "{} · {}",
                job.path.file_name().unwrap().to_str().unwrap(),
                job.file.links.normal_download
            )
        });
    }

    Ok(())
}
