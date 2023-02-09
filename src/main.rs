mod config;
mod patch;
mod sha;

use config::Config;
use patch::{File, Patch, PatchEntry};

use std::cmp::Ordering;

use tokio::{fs, io, time::Instant};

use reqwest::Client;
use toml;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = fs::read_to_string("./config.toml").await?;
    let config: Config = toml::from_str(&config)?;

    let client = Client::builder().gzip(true).build()?;

    let now = Instant::now();

    for game in config.games {
        let name = game.name;
        let mut out = game.output;
        let mut branch = game.branch;
        if out == "" {
            out = format!("games/{}", name.clone())
        }
        if branch == "" {
            branch = "default".into();
        }
        let manifest_url = match game.sandbox {
            true => format!("https://spark-sandbox.gameforge.com/api/v1/patching/download/latest/{}/{}", name, branch),
            false => format!("https://spark.gameforge.com/api/v1/patching/download/latest/{}/{}", name, branch),
        };

        println!("Processing game {}.. (path: {})", name, out);

        fs::create_dir_all(&out).await?;

        let mut manifest = client.get(manifest_url).send().await?.json::<Patch>().await?;

        // sort with folder first
        manifest.entries.sort_by(|l, r| match (l, r) {
            (_, PatchEntry::File(_)) => Ordering::Less,
            (PatchEntry::File(_), _) => Ordering::Greater,
            (_, _) => Ordering::Equal,
        });

        let len = manifest.entries.len();

        // todo: process this in parallel
        for (idx, entry) in manifest.entries.iter().enumerate() {
            let cnt = format!("({}/{}) ", idx, len);
            match entry {
                PatchEntry::Folder(entry) => fs::create_dir_all(format!("{}/{}", out, entry.file)).await?,
                PatchEntry::File(entry) => match fs::File::open(format!("{}/{}", out, entry.file)).await {
                    Ok(file) => match file.metadata().await {
                        Ok(metadata) => {
                            let mut need_dl = false;
                            if metadata.len() != entry.size {
                                println!("{}DL: {} (File length missmatch, {} != {})", cnt, entry.file, metadata.len(), entry.size);
                                need_dl = true;
                            } else if !config.skip_hash {
                                let hash = sha::calc_sha(file).await?;
                                if hash != entry.sha1 {
                                    println!("{}DL: {} (Hash missmatch, {} != {})", cnt, entry.file, hash, entry.sha1);
                                    need_dl = true;
                                } else {
                                    println!("{}OK: {}", cnt, entry.file);
                                }
                            } else {
                                println!("{}OK: {} (hash skipped)", cnt, entry.file);
                            }

                            if need_dl {
                                get(&client, entry, &out).await?;
                            }
                        }
                        Err(_) => {
                            println!("{}DL: {}", cnt, entry.file);
                            get(&client, entry, &out).await?;
                        }
                    },
                    Err(_) => {
                        println!("{}DL: {}", cnt, entry.file);
                        get(&client, entry, &out).await?;
                    }
                },
            }
        }
    }

    println!("All games updated! {}s", now.elapsed().as_secs());

    Ok(())
}

async fn get(client: &Client, entry: &File, out: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut dest = fs::File::create(format!("{}/{}", out, entry.file)).await?;

    let url = format!("http://patches.gameforge.com{}", entry.path);
    let resp = client.get(&url).send().await?;
    let bytes = resp.bytes().await?;

    io::copy(&mut &*bytes, &mut dest).await?;
    dest.sync_all().await?;

    Ok(())
}
