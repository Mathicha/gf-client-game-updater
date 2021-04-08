use std::{fs, io, time::Instant};

mod config;
mod patch;

use config::Config;
use patch::{Patch, PatchEntry};
use reqwest::blocking::Client;
use sha1::{Digest, Sha1};
use toml;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = fs::read_to_string("./config.toml")?;
    let config: Config = toml::from_str(&config)?;

    let client = Client::builder().gzip(true).build()?;

    let now = Instant::now();

    for game in config.games {
        let name = game.name;
        let mut out = game.output;
        if out == "" {
            out = format!("games/{}", name.clone())
        }

        print!("Processing game {}.. (path: {})", name, out);

        fs::create_dir_all(&out)?;

        let mut manifest = client
            .get(&format!(
                "https://spark.gameforge.com/api/v1/patching/download/latest/{}/default",
                name
            ))
            .send()?
            .json::<Patch>()?;

        manifest.entries.sort_by(|a, b| a.file.cmp(&b.file));

        let len = manifest.entries.len();

        for (idx, entry) in manifest.entries.iter().enumerate() {
            if entry.folder {
                fs::create_dir_all(format!("{}/{}", out, entry.file))?;
            } else {
                let cnt = format!("({}/{}) ", idx, len);
                match fs::File::open(format!("{}/{}", out, entry.file)) {
                    Ok(mut file) => match file.metadata() {
                        Ok(metadata) => {
                            if metadata.len() != entry.size {
                                println!(
                                    "{}DL: {} (File length missmatch, {} != {})",
                                    cnt,
                                    entry.file,
                                    metadata.len(),
                                    entry.size
                                );
                                get(&client, entry, &out)?;
                            } else {
                                let mut sha = Sha1::new();
                                io::copy(&mut file, &mut sha)?;
                                let hash = sha.finalize();

                                if format!("{:x}", hash) != entry.sha1 {
                                    println!("{}DL: {} (Hash missmatch, {:x} != {})", cnt, entry.file, hash, entry.sha1);
                                    get(&client, entry, &out)?;
                                } else {
                                    println!("{}OK: {}", cnt, entry.file);
                                }
                            }
                        }
                        Err(_) => {
                            println!("{}DL: {}", cnt, entry.file);
                            get(&client, entry, &out)?;
                        }
                    },
                    Err(_) => {
                        println!("{}DL: {}", cnt, entry.file);
                        get(&client, entry, &out)?;
                    }
                }
            }
        }
    }

    println!("All games updated! {}s", now.elapsed().as_secs());

    Ok(())
}

fn get(client: &Client, entry: &PatchEntry, out: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut dest = fs::File::create(format!("{}/{}", out, entry.file))?;

    let url = format!("http://patches.gameforge.com{}", entry.path);
    client.get(&url).send()?.copy_to(&mut dest)?;

    Ok(())
}
