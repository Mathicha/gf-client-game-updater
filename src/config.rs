use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default)]
    pub games: Vec<Games>,
    #[serde(default)]
    pub skip_hash: bool,
}

#[derive(Deserialize, Debug)]
pub struct Games {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub branch: String,
    #[serde(default)]
    pub token: String,
    #[serde(default)]
    pub output: String,
    #[serde(default)]
    pub sandbox: bool,
}
