use anyhow::Result;
use regex::Regex;
use ring::digest;
use scraper::{Html, Selector};
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn match_mediafire_valid_url(url: &str) -> Option<(String, String)> {
    let re = Regex::new(r"mediafire\.com/(file|file_premium|folder)/(\w+)").unwrap();
    let matches = re.captures(url);
    matches.map(|captures| (captures[1].to_string(), captures[2].to_string()))
}

pub async fn create_directory_if_not_exists(path: &Path) -> Result<()> {
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

pub fn check_hash(file_path: &Path, expected_hash: &String) -> Result<bool, std::io::Error> {
    let mut file = File::open(file_path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;

    let actual_hash = digest::digest(&digest::SHA256, &contents);
    let actual_hash_str = &hex::encode(actual_hash.as_ref());

    Ok(actual_hash_str == expected_hash)
}

#[cfg(test)]
mod tests {
    use crate::global::CLIENT;

    use super::*;

    #[tokio::test]
    async fn test_parse_download_link() {
        let html = &CLIENT
            .get("https://www.mediafire.com/file/tb1d35twcp7oj3p")
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        let link = parse_download_link(html);
        assert!(link.is_some());
    }
}
