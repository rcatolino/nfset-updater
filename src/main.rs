mod config;
mod prefix_iterator;
mod schema;

use anyhow::{Context, Result};
use curl::easy::Easy;
use prefix_iterator::PrefixBatchIterator;
use schema::{Prefix, Response};
use serde_json;
use std::ffi::OsStr;
use std::process::Command;
use std::{env, thread, time};

const API_URL: &'static str = "https://bgp.he.net/super-lg/report/api/v1/prefixes/originated";

fn get_prefixes(asn: &str) -> Result<Vec<Prefix>> {
    let mut buffer = Vec::new();
    let mut easy = Easy::new();
    easy.url(format!("{}/{}", API_URL, asn).as_ref())?;
    easy.get(true)?;
    {
        let mut tx = easy.transfer();
        tx.write_function(|data| {
            buffer.extend_from_slice(data);
            Ok(data.len())
        })?;
        tx.perform()?;
    }

    let resp: Response =
        serde_json::from_slice(buffer.as_slice()).with_context(|| "Parsing GET prefix response")?;
    Ok(resp.prefixes)
}

fn update_set(asn: &str, family: &str, table: &str, name: &str, prefixes: &[&str]) -> Result<()> {
    let prefixes_str = prefixes.join(",");
    println!(
        "nfset AS{} add to {} {} {}: {}",
        asn, family, table, name, prefixes_str
    );

    let status = Command::new("nft")
        .arg("add")
        .arg("element")
        .arg(family)
        .arg(table)
        .arg(name)
        .arg("{")
        .arg(prefixes_str)
        .arg("}")
        .status()?;

    if !status.success() {
        match status.code() {
            Some(code) => Err(anyhow::anyhow!("nft returned with error code {}", code)),
            None => Err(anyhow::anyhow!("nft was killed by signal")),
        }
    } else {
        Ok(())
    }
}

fn main() -> Result<()> {
    let sets = config::parse(env::args().nth(1).as_deref())?;
    for set in sets {
        println!(
            "Enumerating prefixes in set '{}/{}'",
            set.name_ipv4, set.name_ipv6
        );
        for as_number in set.asns {
            let prefixes = match get_prefixes(&as_number) {
                Ok(p) => p,
                Err(e) => {
                    println!("Error retrieving prefixes for AS{}: {}", as_number, e);
                    continue;
                }
            };
            for (batch, is_ipv6) in &mut PrefixBatchIterator::new(prefixes.as_slice()) {
                let set_name = if is_ipv6 {
                    set.name_ipv6.as_str()
                } else {
                    set.name_ipv4.as_str()
                };
                update_set(&as_number, &set.family, &set.table, set_name, batch.as_slice())?
            }
            thread::sleep(time::Duration::from_secs(5));
        }
    }
    Ok(())
}
