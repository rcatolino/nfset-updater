mod schema;

use anyhow::Result;
use curl::easy::Easy;
use schema::{Prefix, Response};
use serde_json;

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

    println!("api response {}", std::str::from_utf8(buffer.as_slice())?);
    let resp: Response = serde_json::from_slice(buffer.as_slice())?;
    Ok(resp.prefixes)
}

fn main() -> Result<()> {
    let p = get_prefixes("3215")?;
    println!("{:?}", p);
    Ok(())
}
