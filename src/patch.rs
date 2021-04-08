use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Patch {
    #[serde(rename = "totalSize")]
    pub total_size: u64,
    pub build: u16,
    pub entries: Vec<PatchEntry>,
}

#[derive(Deserialize, Debug)]
pub struct PatchEntry {
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub sha1: String,
    pub file: String,
    pub flags: u8,
    pub size: u64,
    pub folder: bool,
}
