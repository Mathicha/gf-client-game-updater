use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default)]
    pub games: Vec<Games>,
}

#[derive(Deserialize, Debug)]
pub struct Games {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub output: String,
}
