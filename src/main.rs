mod api;
mod download;
mod global;
mod types;
mod utils;

use crate::api::file;
use crate::api::folder;
use crate::download::{download_file, download_folder};
use crate::types::file_type::FileType;
use crate::utils::{create_directory_if_not_exists, match_mediafire_valid_url};
use anyhow::Result;
use clap::ArgAction;
use clap::{arg, command, value_parser};
use colored::*;
use global::*;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;
use types::client::Client;
use types::download::DownloadJob;

#[tokio::main]
async fn main() -> Result<()> {
    let get_matches = command!()
        .author("NicKoehler")
        .color(clap::ColorChoice::Always)
        .arg(
            arg!([URLS] "List of folders or files to download")
                .required(true)
                .value_parser(value_parser!(String)).num_args(1..),
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
                .value_parser(value_parser!(u32).range(1..=100)),
        )
        .arg(
            arg!(-t --tries <MAX> "Maximum number of tries to repeat for every download")
                .required(false)
                .default_value("1")
                .value_parser(value_parser!(u32).range(1..=10)),
        )
        .arg(
            arg!(-p --proxy <FILE> "Specify a file to read proxies from")
                .required(false)
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            arg!(--"proxy-download" "Downloads files through proxies, the default is to use proxies for the API only")
                .action(ArgAction::SetTrue)
        )
        .get_matches();
    let matches = get_matches;

    let urls: Vec<String> = matches.get_many("URLS").unwrap().cloned().collect();
    let path = matches.get_one::<PathBuf>("output").unwrap().to_path_buf();
    let max = *matches.get_one::<u32>("max").unwrap();
    let tries = *matches.get_one::<u32>("tries").unwrap();
    let proxies: Option<Vec<String>> = matches.get_one::<PathBuf>("proxy").map(|path| {
        let proxy_file = File::open(path).unwrap();
        io::BufReader::new(proxy_file)
            .lines()
            .map_while(Result::ok)
            .collect()
    });
    let proxy_downloads = *matches.get_one::<bool>("proxy-download").unwrap();

    let client = std::sync::Arc::new(Client::new(proxies, proxy_downloads));
    for url in urls.iter() {
        let option = match_mediafire_valid_url(url);

        if let Some(keys) = option {
            TOTAL_PROGRESS_BAR.enable_steady_tick(Duration::from_millis(120));
            TOTAL_PROGRESS_BAR.set_style(PROGRESS_STYLE_TOTAL_START.clone());

            for key in keys.iter() {
                match FileType::from_key(key) {
                    FileType::Folder => {
                        if let Some(folder) = folder::get_info(&client, &key).await?.folder_info {
                            download_folder(
                                &client,
                                &key,
                                path.join(PathBuf::from(folder.name)),
                                1,
                            )
                            .await?;
                        } else {
                            println!(
                                "{}",
                                format!("Warning: Invalid Mediafire folder URL for {}", url)
                                    .yellow()
                            );
                            continue;
                        }
                    }
                    FileType::File => {
                        create_directory_if_not_exists(&path).await?;
                        let response = file::get_info(&key).await?;
                        if let Some(file_info) = response.file_info {
                            let path = path.join(PathBuf::from(&file_info.filename));
                            QUEUE
                                .lock()
                                .await
                                .push(DownloadJob::new(file_info.into(), path));
                        } else {
                            println!(
                                "{}",
                                format!("Warning: Invalid Mediafire file URL for {}", url).yellow()
                            );
                            continue;
                        }
                    }
                    FileType::Invalid => {
                        println!(
                            "{}",
                            format!("Warning: Invalid Mediafire URL: {}", url).yellow()
                        );
                        continue;
                    }
                }
            }
        } else {
            println!(
                "{}",
                format!("Warning: Invalid Mediafire URL: {}", url).yellow()
            );
        }
    }

    if QUEUE.lock().await.is_empty() {
        println!("{}", "Warning: No files to download".yellow());
        return Ok(());
    }

    TOTAL_PROGRESS_BAR.disable_steady_tick();
    TOTAL_PROGRESS_BAR.set_length(QUEUE.lock().await.len() as u64);
    TOTAL_PROGRESS_BAR.set_style(PROGRESS_STYLE_TOTAL_DOWNLOAD.clone());
    TOTAL_PROGRESS_BAR.set_message("Downloading");

    for _ in 0..max {
        let client = client.clone();
        tokio::spawn(async move {
            loop {
                let task = {
                    let mut queue = QUEUE.lock().await;
                    queue.pop()
                };
                if let Some(task) = task {
                    match download_file(&client, &task, tries).await {
                        Ok(_) => SUCCESSFUL_DOWNLOADS.lock().await.push(task),
                        Err(e) => FAILED_DOWNLOADS.lock().await.push((task, e)),
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
                } else {
                    break;
                }
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
    if !failed.is_empty() {
        println!("Failed downloads:");
        failed.iter().for_each(|(job, error)| {
            println!(
                "{}",
                format!(
                    "{} · {} · {}",
                    job.path.file_name().unwrap().to_str().unwrap(),
                    error,
                    job.file.links.normal_download
                )
                .red()
            )
        });
    }

    Ok(())
}
