use crate::types::permissions::Permissions;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Folder {
    pub folderkey: String,
    pub name: String,
    pub description: String,
    pub tags: String,
    pub privacy: String,
    pub created: String,
    pub revision: String,
    pub flag: String,
    pub permissions: Permissions,
    pub file_count: String,
    pub folder_count: String,
    pub dropbox_enabled: String,
    pub created_utc: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FolderInfo {
    pub folderkey: String,
    pub name: String,
    pub description: String,
    pub created: String,
    pub privacy: String,
    pub file_count: String,
    pub folder_count: String,
    pub revision: String,
    pub owner_name: String,
    pub avatar: String,
    pub flag: String,
    pub permissions: Permissions,
    pub created_utc: String,
}
