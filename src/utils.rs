use anyhow::Result;
use futures::StreamExt;
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderValue};
use ring::digest;
use scraper::{Html, Selector};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

use crate::consts::HEADERS;

pub fn match_mediafire_valid_url(url: &str) -> Option<(String, String)> {
    let re = Regex::new(r"mediafire\.com/(file|file_premium|folder)/(\w+)").unwrap();
    let matches = re.captures(url);

    if let Some(captures) = matches {
        Some((captures[1].to_string(), captures[2].to_string()))
    } else {
        None
    }
}

pub async fn save_file(path: &PathBuf, response: reqwest::Response) -> Result<(), anyhow::Error> {
    let mut file = tokio::fs::File::create(path).await?;
    let mut stream = response.bytes_stream();
    Ok(while let Some(chunk) = stream.next().await {
        file.write_all(&chunk?).await?;
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

pub fn check_hash(file_path: &PathBuf, expected_hash: &String) -> Result<bool, std::io::Error> {
    let mut file = File::open(file_path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;

    let actual_hash = digest::digest(&digest::SHA256, &contents);
    let actual_hash_str = &hex::encode(actual_hash.as_ref());

    Ok(actual_hash_str == expected_hash)
}

pub fn build_client() -> reqwest::Client {
    reqwest::Client::builder()
        .default_headers(HeaderMap::from_iter(
            HEADERS
                .iter()
                .map(|(k, v)| (k.clone(), HeaderValue::from_str(v).unwrap())),
        ))
        .build()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_download_link() {
        let client = build_client();

        let html = client
            .get("https://www.mediafire.com/file/tb1d35twcp7oj3p")
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        let link = parse_download_link(&html);
        assert!(link.is_some());
    }
}
