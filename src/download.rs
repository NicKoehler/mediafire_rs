use crate::api::folder::get_content;
use crate::global::*;
use crate::types::client::Client;
use crate::types::download::DownloadJob;
use crate::types::file::File;
use crate::types::folder::Folder;
use crate::types::get_content::Response;
use crate::utils::check_hash;
use crate::utils::{create_directory_if_not_exists, parse_download_link};
use anyhow::{Result, anyhow};
use futures::StreamExt;
use indicatif::ProgressBar;
use reqwest::header::{HeaderMap, RANGE};
use std::io::SeekFrom;
use std::path::PathBuf;
use tokio::fs::remove_file;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use tokio::time::Duration;
use tokio::time::sleep;
use tokio::try_join;

const RETRY_SECS: u64 = 2;

#[async_recursion::async_recursion]
pub async fn download_folder(
    client: &Client,
    folder_key: &str,
    path: PathBuf,
    chunk: u32,
) -> Result<()> {
    create_directory_if_not_exists(&path).await?;
    TOTAL_PROGRESS_BAR.set_message(format!(
        "{}",
        path.components()
            .last()
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap()
    ));

    let (folder_content, file_content) =
        get_folder_and_file_content(client, folder_key, chunk).await?;

    if let Some(files) = file_content.folder_content.files {
        download_files(files, &path).await?;
    }

    if let Some(folders) = folder_content.folder_content.folders {
        download_folders(client, folders, &path, chunk).await?;
    }

    if folder_content.folder_content.more_chunks == "yes"
        || file_content.folder_content.more_chunks == "yes"
    {
        download_folder(client, folder_key, path, chunk + 1).await?;
    }

    Ok(())
}

async fn get_folder_and_file_content(
    client: &Client,
    folder_key: &str,
    chunk: u32,
) -> Result<(Response, Response)> {
    match try_join!(
        get_content(client, folder_key, "folders", chunk),
        get_content(client, folder_key, "files", chunk)
    ) {
        Ok((folder_content, file_content)) => Ok((folder_content, file_content)),
        Err(_) => Err(anyhow!("Invalid Mediafire URL")),
    }
}

async fn download_files(files: Vec<File>, path: &PathBuf) -> Result<()> {
    files.iter().for_each(|file| {
        let file_path = path.join(&file.filename);
        let download_job = DownloadJob::new(file.clone(), file_path);
        QUEUE.push(download_job);
    });
    Ok(())
}

async fn download_folders(
    client: &Client,
    folders: Vec<Folder>,
    path: &PathBuf,
    chunk: u32,
) -> Result<()> {
    for folder in folders {
        let folder_path = path.join(&folder.name);
        if let Err(e) = download_folder(client, &folder.folderkey, folder_path, chunk).await {
            return Err(e);
        }
    }
    Ok(())
}

pub async fn download_file(
    client: &Client,
    download_job: &DownloadJob,
    mut tries: u32,
) -> Result<()> {
    let bar = MULTI_PROGRESS_BAR.insert_from_back(1, ProgressBar::new(0));
    let filename = file_name(download_job);

    let mut last_error: anyhow::Error = anyhow!("Something went wrong");

    while tries > 0 {
        setup_bar(&bar, &filename);

        match attempt_download(client, download_job, &bar).await {
            Ok(()) => {
                bar.set_prefix(filename);
                bar.set_style(PROGRESS_STYLE.clone());
                bar.abandon_with_message("✅");
                return Ok(());
            }
            Err(e) => {
                tries -= 1;
                last_error = e;
                bar.set_style(PROGRESS_STYLE_ERROR.clone());
                bar.set_message("🔃");
                if tries > 0 {
                    bar.set_prefix(format!("🔃 Retrying... {} left", tries));
                } else {
                    bar.set_prefix("❌ No more retries left");
                }
                sleep(Duration::from_secs(RETRY_SECS)).await;
            }
        }
    }
    bar.set_prefix(filename);
    bar.set_style(PROGRESS_STYLE_ERROR.clone());
    bar.abandon_with_message("❌");
    Err(last_error)
}

async fn attempt_download(
    client: &Client,
    download_job: &DownloadJob,
    bar: &ProgressBar,
) -> Result<()> {
    use reqwest::StatusCode;

    let filename = file_name(download_job);
    let resume_info = prepare_resume(download_job, bar).await?;
    if resume_info.is_none() {
        return Ok(());
    }
    let (resume, start_bytes) = resume_info.unwrap();

    bar.set_message("🌀");
    bar.set_prefix(if resume {
        "Resuming download..."
    } else {
        "Getting download link..."
    });

    let response = get_response(client, download_job, resume, start_bytes)
        .await?
        .ok_or_else(|| anyhow!("Error getting download link, check the link in the browser"))?;

    if resume && response.status() != StatusCode::PARTIAL_CONTENT {
        remove_file(&download_job.path).await.ok();
    }

    if !response.status().is_success() {
        remove_file(&download_job.path).await.ok();
        return Err(anyhow!(response.status()));
    }

    bar.set_prefix(filename.clone());

    if let Err(e) = stream_file_to_disk(&download_job.path, response, bar, resume).await {
        remove_file(&download_job.path).await.ok();
        return Err(anyhow!(e));
    }

    bar.set_message("💾 ");
    bar.set_prefix("Checking hash...");

    if !check_hash(&download_job.path, &download_job.file.hash)? {
        remove_file(&download_job.path).await.ok();
        return Err(anyhow!("File downloaded, but possibly corrupted"));
    }

    Ok(())
}

pub async fn stream_file_to_disk(
    path: &PathBuf,
    response: reqwest::Response,
    progress_bar: &ProgressBar,
    resume_download: bool,
) -> Result<(), anyhow::Error> {
    progress_bar.set_style(PROGRESS_STYLE_DOWNLOAD.clone());
    progress_bar.set_message("🔽");
    let content_length = response.content_length().unwrap_or(0);
    let mut start_bytes = 0;

    let mut file = if resume_download {
        let mut f = tokio::fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(path)
            .await?;

        start_bytes = f.seek(SeekFrom::End(0)).await?;
        progress_bar.inc(start_bytes);
        f
    } else {
        tokio::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .await?
    };

    progress_bar.set_length(content_length + start_bytes);

    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        progress_bar.inc(chunk.len() as u64);
        file.write_all(&chunk).await?;
    }
    file.flush().await?;
    Ok(())
}

fn file_name(download_job: &DownloadJob) -> String {
    download_job
        .path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("unknown")
        .to_string()
}

fn setup_bar(bar: &ProgressBar, filename: &str) {
    bar.reset();
    bar.set_style(PROGRESS_STYLE.clone());
    bar.set_prefix(filename.to_string());
}

async fn prepare_resume(
    download_job: &DownloadJob,
    bar: &ProgressBar,
) -> Result<Option<(bool, u64)>> {
    if !download_job.path.exists() {
        return Ok(Some((false, 0)));
    }

    bar.set_message("💾 Checking existing file...");

    if check_hash(&download_job.path, &download_job.file.hash)? {
        return Ok(None);
    }

    let mut file = tokio::fs::File::open(&download_job.path).await?;
    let size = file.seek(SeekFrom::End(0)).await?;
    Ok(Some((size > 0, size)))
}

async fn get_response(
    client: &Client,
    download_job: &DownloadJob,
    resume: bool,
    start_bytes: u64,
) -> Result<Option<reqwest::Response>> {
    let mut headers = HeaderMap::new();
    if resume {
        headers.insert(RANGE, format!("bytes={}-", start_bytes).parse()?);
    }

    let response = client
        .api_client
        .get(&download_job.file.links.normal_download)
        .headers(headers.clone())
        .send()
        .await?;

    let is_html = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.starts_with("text/html"))
        .unwrap_or(false);

    if !is_html {
        return Ok(Some(response));
    }

    let body = response.text().await?;
    let link = parse_download_link(&body);

    Ok(match link {
        Some(link) => Some(
            client
                .download_client
                .get(link)
                .headers(headers)
                .send()
                .await?,
        ),
        None => None,
    })
}
