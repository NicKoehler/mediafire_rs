use crate::types::{get_content, get_info};
use reqwest::get;

const BASE_URL_GET_INFO: &str =
    "https://www.mediafire.com/api/1.5/folder/get_info.php?response_format=json&folder_key=";
const BASE_URL_GET_CONTENT: &str =
    "https://www.mediafire.com/api/1.5/folder/get_content.php?response_format=json&folder_key=";

pub async fn get_content(
    folder_key: &str,
    content_type: &str,
    chunk: u32,
) -> Result<get_content::Response, reqwest::Error> {
    return get(format!(
        "{BASE_URL_GET_CONTENT}{folder_key}&content_type={content_type}&chunk={chunk}"
    ))
    .await?
    .json::<get_content::Root>()
    .await
    .map(|root| root.response);
}

pub async fn get_info(folder_key: &str) -> Result<get_info::Response, reqwest::Error> {
    return get(format!("{BASE_URL_GET_INFO}{folder_key}"))
        .await?
        .json::<get_info::Root>()
        .await
        .map(|root| root.response);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_folders_content() {
        let response = get_content("xqub019s2e2l1", "folders", 1).await.unwrap();
        assert_eq!(response.folder_content.folderkey, "xqub019s2e2l1");
    }

    #[tokio::test]
    async fn test_get_files_content() {
        let response = get_content("xqub019s2e2l1", "files", 1).await.unwrap();
        assert_eq!(response.folder_content.folderkey, "xqub019s2e2l1");
    }

    #[tokio::test]
    async fn test_get_info() {
        let response = get_info("xqub019s2e2l1").await.unwrap();
        assert!(response.folder_info.is_some());
    }
}
