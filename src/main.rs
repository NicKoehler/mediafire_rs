mod api;
mod download;
mod types;
mod utils;

use crate::api::file;
use crate::api::folder;
use crate::download::{download_file, download_folder};
use crate::utils::{create_directory_if_not_exists, match_mediafire_valid_url};
use anyhow::{anyhow, Result};
use clap::{arg, command, value_parser};

use std::path::PathBuf;

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
        .get_matches();

    let url = matches.get_one::<String>("URL").unwrap();
    let path = matches.get_one::<PathBuf>("output").unwrap().to_path_buf();
    let option = match_mediafire_valid_url(url);

    if let Some((mode, key)) = option {
        if mode == "folder" {
            let response = folder::get_info(&key).await;
            if let Ok(response) = response {
                if let Some(folder) = response.folder_info {
                    download_folder(&key, path.join(PathBuf::from(folder.name)), 1).await?;
                    return Ok(());
                }
            }
        } else {
            create_directory_if_not_exists(&path).await?;
            let response = file::get_info(&key).await;
            if let Ok(response) = response {
                if let Some(file_info) = response.file_info {
                    let path = path.join(PathBuf::from(&file_info.filename));
                    download_file(&file_info.into(), path).await?;
                    return Ok(());
                }
            }
        }
    }
    return Err(anyhow!("Invalid Mediafire URL"));
}
