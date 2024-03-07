use crate::api::folder::get_content;
use crate::types::file::File;
use crate::types::folder::Folder;
use crate::types::get_content::Response;
use crate::utils::check_hash;
use crate::utils::{create_directory_if_not_exists, parse_download_link, save_file};
use anyhow::{anyhow, Result};
use colored::*;
use futures::future::join_all;
use reqwest::get;
use std::path::PathBuf;
use tokio::try_join;

#[async_recursion::async_recursion]
pub async fn download_folder(folder_key: &str, path: PathBuf, chunk: u32) -> Result<()> {
    create_directory_if_not_exists(&path).await?;

    println!(
        "{}",
        format!("[INFO] Downloading folder {}", get_bold_folder_name(&path)).blue()
    );

    let (folder_content, file_content) = get_folder_and_file_content(folder_key, chunk).await?;

    if let Some(files) = file_content.folder_content.files {
        download_files(files, &path).await?;
    }

    if let Some(folders) = folder_content.folder_content.folders {
        download_folders(folders, &path, chunk).await?;
    }

    if folder_content.folder_content.more_chunks == "yes"
        || file_content.folder_content.more_chunks == "yes"
    {
        download_folder(folder_key, path, chunk + 1).await?;
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
        Err(_) => Err(anyhow!("[ERROR] Invalid Mediafire URL")),
    }
}

async fn download_files(files: Vec<File>, path: &PathBuf) -> Result<()> {
    let download_futures = files.iter().map(|file| {
        let file_path = path.join(&file.filename);
        download_file(&file, file_path)
    });

    if download_futures.len() > 0 {
        println!(
            "{}",
            format!(
                "[INFO] Downloading {} files",
                download_futures.len().to_string().bold()
            )
            .blue()
        );
        join_all(download_futures).await;
    }

    Ok(())
}

async fn download_folders(folders: Vec<Folder>, path: &PathBuf, chunk: u32) -> Result<()> {
    let download_futures = folders.iter().map(|folder| {
        let folder_path = path.join(&folder.name);
        download_folder(&folder.folderkey, folder_path, chunk)
    });

    join_all(download_futures).await;

    Ok(())
}

pub async fn download_file(file: &File, path: PathBuf) -> Result<()> {
    if path.is_file() {
        if check_hash(&path, &file.hash)? {
            println!(
                "{}",
                format!("[WARN] File {} already exists", get_bold_file_name(&path)).yellow()
            );
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

    println!(
        "{}",
        format!("[INFO] Downloading file {}", get_bold_file_name(&path)).blue()
    );

    let body = get(&file.links.normal_download).await?.text().await?;

    if let Some(link) = parse_download_link(&body) {
        let response = get(link).await?;
        match save_file(&path, response).await {
            Ok(_) => {
                println!(
                    "{}",
                    format!("[INFO] File {:?} downloaded", path.file_name().unwrap()).green()
                );
                Ok(())
            }
            Err(e) => Err(e),
        }
    } else {
        Err(anyhow!("[ERROR] Invalid download link"))
    }
}
