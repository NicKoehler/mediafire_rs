use crate::types::file::File;
use crate::types::folder::Folder;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub action: String,
    pub asynchronous: String,
    pub folder_content: FolderContent,
    pub result: String,
    pub current_api_version: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FolderContent {
    pub chunk_size: String,
    pub content_type: String,
    pub chunk_number: String,
    pub folderkey: String,
    pub folders: Option<Vec<Folder>>,
    pub files: Option<Vec<File>>,
    pub more_chunks: String,
    pub revision: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Permissions {
    pub value: String,
    pub explicit: String,
    pub read: String,
    pub write: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Root {
    pub response: Response,
}
