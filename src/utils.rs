use anyhow::Result;
use regex::Regex;
use ring::digest;
use scraper::{Html, Selector};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

pub fn match_mediafire_valid_url(url: &str) -> Option<(String, String)> {
    let re = Regex::new(r"mediafire\.com/(file|folder)/(\w+)").unwrap();
    let matches = re.captures(url);

    if let Some(captures) = matches {
        Some((captures[1].to_string(), captures[2].to_string()))
    } else {
        None
    }
}

pub async fn save_file(
    path: &PathBuf,
    mut response: reqwest::Response,
) -> Result<(), anyhow::Error> {
    let mut file = tokio::fs::File::create(path).await?;
    Ok(while let Some(chunk) = response.chunk().await? {
        file.write_all(&chunk).await?;
    })
}

pub async fn create_directory_if_not_exists(path: &PathBuf) -> Result<()> {
    if !path.exists() {
        tokio::fs::create_dir_all(&path).await?;
    }
    Ok(())
}

pub fn parse_download_link(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("#downloadButton").ok()?;
    let link = document
        .select(&selector)
        .next()?
        .value()
        .attr("href")?
        .to_string();
    Some(link)
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::blocking::get;
    #[test]
    fn test_parse_download_link() {
        let html = get("https://www.mediafire.com/file/9qkaxfrmd78vyj3/Recovery.exe/file")
            .unwrap()
            .text()
            .unwrap();
        let link = parse_download_link(&html);
        assert!(link.is_some());
    }
}

pub fn check_hash(file_path: &PathBuf, expected_hash: &String) -> Result<bool, std::io::Error> {
    let mut file = File::open(file_path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;

    let actual_hash = digest::digest(&digest::SHA256, &contents);
    let actual_hash_str = &hex::encode(actual_hash.as_ref());

    Ok(actual_hash_str == expected_hash)
}
