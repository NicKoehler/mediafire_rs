use crate::api::folder::get_content;
use crate::types::download::DownloadJob;
use crate::types::file::File;
use crate::types::folder::Folder;
use crate::types::get_content::Response;
use crate::utils::{build_client, check_hash};
use crate::utils::{create_directory_if_not_exists, parse_download_link, save_file};
use anyhow::{anyhow, Result};
use deadqueue::unlimited::Queue;
use indicatif::{MultiProgress, ProgressBar, ProgressFinish, ProgressStyle};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::try_join;

#[async_recursion::async_recursion]
pub async fn download_folder(
    folder_key: &str,
    path: PathBuf,
    chunk: u32,
    queue: Arc<Queue<DownloadJob>>,
    progress_bar: Arc<ProgressBar>,
) -> Result<()> {
    create_directory_if_not_exists(&path).await?;
    progress_bar.set_message(format!(
        "{}",
        path.components()
            .last()
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap()
    ));

    let (folder_content, file_content) = get_folder_and_file_content(folder_key, chunk).await?;

    if let Some(files) = file_content.folder_content.files {
        download_files(files, &path, queue.clone()).await?;
    }

    if let Some(folders) = folder_content.folder_content.folders {
        download_folders(folders, &path, chunk, queue.clone(), progress_bar.clone()).await?;
    }

    if folder_content.folder_content.more_chunks == "yes"
        || file_content.folder_content.more_chunks == "yes"
    {
        download_folder(
            folder_key,
            path,
            chunk + 1,
            queue.clone(),
            progress_bar.clone(),
        )
        .await?;
    }

    Ok(())
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

async fn download_files(
    files: Vec<File>,
    path: &PathBuf,
    queue: Arc<Queue<DownloadJob>>,
) -> Result<()> {
    files.iter().for_each(|file| {
        let file_path = path.join(&file.filename);
        let download_job = DownloadJob::new(file.clone(), file_path);
        queue.push(download_job);
    });
    Ok(())
}

async fn download_folders(
    folders: Vec<Folder>,
    path: &PathBuf,
    chunk: u32,
    queue: Arc<Queue<DownloadJob>>,
    progress_bar: Arc<ProgressBar>,
) -> Result<()> {
    for folder in folders {
        let folder_path = path.join(&folder.name);
        if let Err(e) = download_folder(
            &folder.folderkey,
            folder_path,
            chunk,
            queue.clone(),
            progress_bar.clone(),
        )
        .await
        {
            progress_bar.set_message(format!("Error: {}", e));
        }
    }
    Ok(())
}

pub async fn download_file(
    download_job: DownloadJob,
    multi_progress_bar: &MultiProgress,
) -> Result<()> {
    let bar = multi_progress_bar
        .insert_from_back(1, ProgressBar::new(0))
        .with_finish(ProgressFinish::AndLeave);
    bar.set_style(
        ProgressStyle::default_bar()
            .template(&format!(
                "[{{bar:30}}] {{msg}} · {}",
                download_job.path.file_name().unwrap().to_str().unwrap()
            ))
            .unwrap()
            .progress_chars("=>-"),
    );
    bar.set_message("Getting download link...");

    if download_job.path.is_file() {
        if check_hash(&download_job.path, &download_job.file.hash)? {
            bar.finish_with_message("Already downloaded 🎉");
            return Ok(());
        }
    }

    let client = build_client();
    let response = {
        let response = client
            .get(&download_job.file.links.normal_download)
            .send()
            .await?;
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
        if let Err(_) = save_file(&download_job.path, response, &bar).await {
            bar.finish_with_message("Failed to download ❌");
            return Err(anyhow!("Invalid download link"));
        }
    }
    Ok(())
}
