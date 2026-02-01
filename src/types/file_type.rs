pub enum FileType {
    File,
    Folder,
    Invalid,
}

impl FileType {
    pub fn from_key(key: &str) -> Self {
        match key.len() {
            15 => FileType::File,
            13 => FileType::Folder,
            _ => FileType::Invalid,
        }
    }
}
