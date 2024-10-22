use reqwest::header::{HeaderName, ACCEPT_ENCODING, USER_AGENT};

pub const HEADERS : &[(HeaderName, &str)] = &[
    (USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.90 Safari/537.36"),
    (ACCEPT_ENCODING, "gzip"),
];
