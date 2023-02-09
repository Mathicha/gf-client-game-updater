use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Patch {
    #[serde(rename = "totalSize")]
    pub total_size: u64,
    pub build: u16,
    pub entries: Vec<PatchEntry>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum PatchEntry {
    File(File),
    Folder(Folder),
}

#[derive(Deserialize, Debug)]
pub struct File {
    pub path: String,
    pub sha1: String,
    pub file: String,
    pub flags: u8,
    pub size: u64,
    pub folder: bool,
}

#[derive(Deserialize, Debug)]
pub struct Folder {
    pub file: String,
    pub flags: u8,
    pub size: u64,
    pub folder: bool,
}
