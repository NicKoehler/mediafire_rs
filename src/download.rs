use crate::api::folder::get_content;
use crate::types::file::File;
use crate::types::folder::Folder;
use crate::types::get_content::Response;
use crate::utils::{build_client, check_hash};
use crate::utils::{create_directory_if_not_exists, parse_download_link, save_file};
use anyhow::{anyhow, Result};
use colored::*;
use futures::future::join_all;
use indicatif::MultiProgress;
use std::path::PathBuf;
use tokio::try_join;

#[async_recursion::async_recursion]
pub async fn download_folder(
    folder_key: &str,
    path: PathBuf,
    chunk: u32,
    max: usize,
) -> Result<()> {
    create_directory_if_not_exists(&path).await?;

    println!(
        "{}",
        format!("[INFO] Downloading folder {}", get_bold_folder_name(&path)).blue()
    );

    let (folder_content, file_content) = get_folder_and_file_content(folder_key, chunk).await?;

    if let Some(files) = file_content.folder_content.files {
        download_files(files, &path, max).await?;
    }

    if let Some(folders) = folder_content.folder_content.folders {
        download_folders(folders, &path, chunk, max).await?;
    }

    if folder_content.folder_content.more_chunks == "yes"
        || file_content.folder_content.more_chunks == "yes"
    {
        download_folder(folder_key, path, chunk + 1, max).await?;
    }

    Ok(())
}

fn get_bold_folder_name(path: &PathBuf) -> ColoredString {
    path.components()
        .last()
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap()
        .bold()
}

fn get_bold_file_name(path: &PathBuf) -> ColoredString {
    path.file_name().unwrap().to_str().unwrap().bold()
}

async fn get_folder_and_file_content(folder_key: &str, chunk: u32) -> Result<(Response, Response)> {
    match try_join!(
        get_content(folder_key, "folders", chunk),
        get_content(folder_key, "files", chunk)
    ) {
        Ok((folder_content, file_content)) => Ok((folder_content, file_content)),
        Err(_) => Err(anyhow!("Invalid Mediafire URL")),
    }
}

async fn download_files(files: Vec<File>, path: &PathBuf, max: usize) -> Result<()> {
    if files.len() == 0 {
        return Ok(());
    }

    let mut count = 0;

    let progress_bar = MultiProgress::new();

    for files_chunk in files.chunks(max) {
        count += files_chunk.len();
        println!(
            "{}",
            format!(
                "[INFO] Downloading {}/{} files from folder {}",
                count,
                files.len(),
                get_bold_folder_name(path)
            )
            .blue()
        );
        let download_futures = files_chunk.iter().map(|file| {
            let file_path = path.join(&file.filename);
            download_file(&file, file_path, &progress_bar)
        });
        join_all(download_futures).await;
    }
    Ok(())
}

async fn download_folders(
    folders: Vec<Folder>,
    path: &PathBuf,
    chunk: u32,
    max: usize,
) -> Result<()> {
    for folder in folders {
        let folder_path = path.join(&folder.name);
        if let Err(e) = download_folder(&folder.folderkey, folder_path, chunk, max).await {
            println!("{}", format!("[WARN] {}", e).yellow());
        }
    }
    Ok(())
}

pub async fn download_file(
    file: &File,
    path: PathBuf,
    multi_progress_bar: &MultiProgress,
) -> Result<()> {
    if path.is_file() {
        if check_hash(&path, &file.hash)? {
            return Ok(());
        }
        println!(
            "{}",
            format!(
                "[WARN] File {} currupted, overwriting",
                get_bold_file_name(&path)
            )
            .yellow()
        );
    }

    let client = build_client();
    let response = {
        let response = client.get(&file.links.normal_download).send().await?;
        if response.headers().get("content-type").unwrap() == &"text/html; charset=UTF-8" {
            if let Some(link) = parse_download_link(&response.text().await?) {
                Some(client.get(link).send().await?)
            } else {
                None
            }
        } else {
            Some(response)
        }
    };

    if let Some(response) = response {
        match save_file(&path, response, &multi_progress_bar).await {
            Err(e) => {
                println!(
                    "{}",
                    format!("[ERROR] Failed to download {}", get_bold_file_name(&path)).red()
                );
                Err(e)
            }
            _ => Ok(()),
        }
    } else {
        println!(
            "{}",
            format!("[ERROR] Failed to download {}", get_bold_file_name(&path)).red()
        );
        Err(anyhow!("Invalid download link"))
    }
}
