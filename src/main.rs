mod schema;
mod config;

use anyhow::Result;
use curl::easy::Easy;
use schema::{Prefix, Response};
use serde_json;
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

fn main() -> Result<()> {
    let sets = config::parse(env::args().nth(1).as_deref())?;
    for set in sets {
        println!("Enumerating prefixes in set '{}'", set.name);
        for as_number in set.asns {
            for prefix in get_prefixes(&as_number)? {
                println!("{}:{}: {}", set.name, as_number, prefix.prefix);
            }
            thread::sleep(time::Duration::from_secs(5));
        }
    }
    Ok(())
}
