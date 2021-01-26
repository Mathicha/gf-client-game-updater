use std::{fs, io};

use reqwest::{blocking::Client, header};
use serde::Deserialize;
use sha1::{Digest, Sha1};

#[derive(Deserialize)]
struct Patch {
    #[serde(rename = "totalSize")]
    total_size: u64,
    build: u16,
    entries: Vec<PatchEntry>,
}

#[derive(Deserialize)]
struct PatchEntry {
    #[serde(default)]
    path: String,
    #[serde(default)]
    sha1: String,
    file: String,
    flags: u8,
    size: u64,
    folder: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut headers = header::HeaderMap::new();
    headers.insert("Accept-Encoding", header::HeaderValue::from_static("gzip"));

    let client = Client::builder().default_headers(headers).gzip(true).build()?;

    let mut manifest = client
        .get("https://spark.gameforge.com/api/v1/patching/download/latest/tera/default")
        .send()?
        .json::<Patch>()?;

    manifest.entries.sort_by(|a, b| a.file.cmp(&b.file));

    let len = manifest.entries.len();

    for (idx, entry) in manifest.entries.iter().enumerate() {
        if entry.folder {
            fs::create_dir_all(&entry.file)?;
        } else {
            let cnt = format!("{}/{}", idx, len);
            match fs::File::open(&entry.file) {
                Ok(mut file) => match file.metadata() {
                    Ok(metadata) => {
                        if metadata.len() != entry.size {
                            println!(
                                "({}) DL: {} (File length missmatch, {} != {})",
                                cnt,
                                entry.file,
                                metadata.len(),
                                entry.size
                            );
                            get(&client, entry)?;
                        } else {
                            let mut sha = Sha1::new();
                            io::copy(&mut file, &mut sha)?;
                            let hash = sha.finalize();

                            if format!("{:x}", hash) != entry.sha1 {
                                println!("({}) DL: {} (Hash missmatch, {:x} != {})", cnt, entry.file, hash, entry.sha1);
                                get(&client, entry)?;
                            } else {
                                println!("({}) OK: {}", cnt, entry.file);
                            }
                        }
                    }
                    Err(_) => {
                        println!("({}) DL: {}", cnt, entry.file);
                        get(&client, entry)?;
                    }
                },
                Err(_) => {
                    println!("({}) DL: {}", cnt, entry.file);
                    get(&client, entry)?;
                }
            }
        }
    }

    Ok(())
}

fn get(client: &Client, entry: &PatchEntry) -> Result<(), Box<dyn std::error::Error>> {
    let mut dest = fs::File::create(&entry.file)?;

    let url = format!("http://patches.gameforge.com{}", entry.path);
    client.get(&url).send()?.copy_to(&mut dest)?;

    Ok(())
}
