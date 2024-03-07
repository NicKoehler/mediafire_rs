use crate::types::file::FileInfo;
use crate::types::folder::FolderInfo;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Root {
    pub response: Response,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub action: String,
    pub folder_info: Option<FolderInfo>,
    pub file_info: Option<FileInfo>,
    pub result: String,
    pub current_api_version: String,
}
