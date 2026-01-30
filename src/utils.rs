use anyhow::Result;
use regex::Regex;
use ring::digest::{Context, SHA256};
use scraper::{Html, Selector};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

pub fn match_mediafire_valid_url(url: &str) -> Option<Vec<String>> {
    let re = Regex::new(r"mediafire\.com/(file|file_premium|folder|download)/([\w,]+)").unwrap();
    let matches = re.captures(url);

    if let Some(captures) = matches {
        Some(captures[2].split(',').map(|t| t.to_string()).collect())
    } else {
        None
    }
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

pub fn check_hash(file_path: &PathBuf, expected_hash: &str) -> Result<bool, std::io::Error> {
    let expected = expected_hash.trim().to_lowercase();

    let mut file = File::open(file_path)?;
    let mut buffer = [0u8; 8192];

    let actual_hash_hex = match expected.len() {
        32 => {
            // MD5
            let mut context = md5::Context::new();
            loop {
                let count = file.read(&mut buffer)?;
                if count == 0 {
                    break;
                }
                context.consume(&buffer[..count]);
            }
            format!("{:x}", context.finalize())
        }
        64 => {
            // SHA-256
            let mut context = Context::new(&SHA256);
            loop {
                let count = file.read(&mut buffer)?;
                if count == 0 {
                    break;
                }
                context.update(&buffer[..count]);
            }
            hex::encode(context.finish().as_ref())
        }
        _ => {
            return Ok(false);
        }
    };

    Ok(actual_hash_hex == expected)
}

#[cfg(test)]
mod tests {
    use crate::types::client::Client;

    use super::*;

    #[tokio::test]
    async fn test_parse_download_link() {
        let client = Client::new(None, false);
        let html = client
            .api_client
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
