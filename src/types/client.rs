use reqwest::header::{HeaderMap, HeaderValue, ACCEPT_ENCODING, USER_AGENT};

pub struct Client {
    /// Client for sending requests to the mediafire API
    pub api_client: reqwest::Client,
    /// Client for downloading files
    pub download_client: reqwest::Client,
}

impl Client {
    pub fn new(proxies: Option<Vec<String>>, proxy_downloads: bool) -> Self {
        let mut api_builder = reqwest::Client::builder()
            .use_rustls_tls()
            .default_headers(HeaderMap::from_iter([
                    (USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.90 Safari/537.36")),
                    (ACCEPT_ENCODING, HeaderValue::from_static("gzip")),
            ]));
        let mut download_builder = reqwest::Client::builder()
            .use_rustls_tls()
            .default_headers(HeaderMap::from_iter([
                    (USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.90 Safari/537.36")),
                    (ACCEPT_ENCODING, HeaderValue::from_static("gzip")),
            ]));

        for proxy in proxies.into_iter().flatten() {
            api_builder = api_builder.proxy(reqwest::Proxy::all(&proxy).unwrap());
            if proxy_downloads {
                download_builder = download_builder.proxy(reqwest::Proxy::all(&proxy).unwrap());
            }
        }

        Self {
            api_client: api_builder.build().unwrap(),
            download_client: download_builder.build().unwrap()
        }
    }
}
