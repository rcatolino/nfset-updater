use std::fs::File;

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_yaml;

#[derive(Deserialize, Debug)]
pub struct Set {
    pub name_ipv4: String,
    pub name_ipv6: String,
    pub asns: Vec<String>,
    pub family: String,
    pub table: String,
}

pub fn parse(path: Option<&str>) -> Result<Vec<Set>> {
    let path = path.unwrap_or("config.yml");
    serde_yaml::from_reader(File::open(path).with_context(|| format!("Opening config '{}'", path))?)
        .with_context(|| format!("Parsing config '{}'", path))
        .map_err(|r| r.into())
}
