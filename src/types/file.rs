use crate::types::permissions::Permissions;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct File {
    pub quickkey: String,
    pub hash: String,
    pub filename: String,
    pub description: String,
    pub size: String,
    pub privacy: String,
    pub created: String,
    pub password_protected: String,
    pub mimetype: String,
    pub filetype: String,
    pub view: String,
    pub edit: String,
    pub revision: String,
    pub flag: String,
    pub permissions: Permissions,
    pub downloads: String,
    pub views: String,
    pub links: Links,
    pub created_utc: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Links {
    pub normal_download: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileInfo {
    pub quickkey: String,
    pub filename: String,
    pub ready: String,
    pub created: String,
    pub description: String,
    pub size: String,
    pub privacy: String,
    pub password_protected: String,
    pub hash: String,
    pub filetype: String,
    pub mimetype: String,
    pub owner_name: String,
    pub flag: String,
    pub permissions: Permissions,
    pub revision: String,
    pub view: String,
    pub edit: String,
    pub links: Links,
    pub created_utc: String,
}

impl From<FileInfo> for File {
    fn from(file_info: FileInfo) -> Self {
        File {
            quickkey: file_info.quickkey,
            hash: file_info.hash,
            filename: file_info.filename,
            description: file_info.description,
            size: file_info.size,
            privacy: file_info.privacy,
            created: file_info.created,
            password_protected: file_info.password_protected,
            mimetype: file_info.mimetype,
            filetype: file_info.filetype,
            view: file_info.view,
            edit: file_info.edit,
            revision: file_info.revision,
            flag: file_info.flag,
            permissions: file_info.permissions,
            downloads: "".to_string(),
            views: "".to_string(),
            links: file_info.links,
            created_utc: file_info.created_utc,
        }
    }
}
