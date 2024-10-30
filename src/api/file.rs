use crate::types::get_info;
use reqwest::get;

const BASE_URL_GET_INFO: &str =
    "https://www.mediafire.com/api/1.5/file/get_info.php?response_format=json&quick_key=";

pub async fn get_info(file_key: &str) -> Result<get_info::Response, reqwest::Error> {
    get(format!("{BASE_URL_GET_INFO}{file_key}"))
        .await?
        .json::<get_info::Root>()
        .await
        .map(|root| root.response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_info() {
        let response = get_info("v91fqemfiau67jr").await.unwrap();
        assert!(response.file_info.is_some());
    }
}
