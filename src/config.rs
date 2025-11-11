use std::fs::File;

use anyhow::Result;
use serde::Deserialize;
use serde_yaml;

#[derive(Deserialize, Debug)]
pub struct Set {
    pub name: String,
    pub asns: Vec<String>,
}

pub fn parse(path: Option<&str>) -> Result<Vec<Set>> {
    let conf = serde_yaml::from_reader(
        File::open(path.unwrap_or("config.yml"))?
    )?;
    Ok(conf)
}
