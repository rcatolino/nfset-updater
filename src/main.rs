mod config;
mod schema;

use anyhow::Result;
use curl::easy::Easy;
use schema::{Prefix, Response};
use serde_json;
use std::net;
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

    let resp: Response = serde_json::from_slice(buffer.as_slice())?;
    Ok(resp.prefixes)
}

fn is_ipv6(prefix: &str) -> Result<bool> {
    let ip = prefix
        .splitn(2, "/")
        .nth(0)
        .ok_or(anyhow::anyhow!("Missing netmask in prefix {}", prefix))?;
    let parsed: net::IpAddr = ip.parse()?;
    Ok(parsed.is_ipv6())
}

fn update_set(family: &str, table: &str, name: &str, prefix: &str) -> Result<()> {
    // TODO: batch updates
    let status = Command::new("nft")
        .arg("add")
        .arg("element")
        .arg(family)
        .arg(table)
        .arg(name)
        .args(["{", prefix, "}"])
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
            for prefix in get_prefixes(&as_number)? {
                let set_name = if is_ipv6(prefix.prefix.as_str())? {
                    set.name_ipv6.as_str()
                } else {
                    set.name_ipv4.as_str()
                };
                println!(
                    "nft add element {} {} {}",
                    set.table, set_name, prefix.prefix
                );
                update_set(&set.family, &set.table, set_name, &prefix.prefix)?
            }
            thread::sleep(time::Duration::from_secs(5));
        }
    }
    Ok(())
}
