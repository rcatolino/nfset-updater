use anyhow::Result;
use std::net;

use crate::schema::Prefix;

pub struct PrefixBatchIterator<'a> {
    remainder: usize,
    data: &'a [Prefix],
}

impl<'a> PrefixBatchIterator<'a> {
    pub(crate) fn new(prefixes: &'a [Prefix]) -> Self {
        Self {
            remainder: prefixes.len(),
            data: prefixes,
        }
    }
}

fn is_ipv6(prefix: &str) -> Result<bool> {
    if prefix.ends_with("/0") {
        return Err(anyhow::anyhow!("Bogus prefix {}", prefix));
    }

    let ip = prefix
        .splitn(2, "/")
        .nth(0)
        .ok_or(anyhow::anyhow!("Missing netmask in prefix {}", prefix))?;
    let parsed: net::IpAddr = ip.parse()?;
    Ok(parsed.is_ipv6())
}

// this ugly iterator returns batch of prefixes of the same type (ipv4/ipv6)
// with a max size of 20 elements
impl<'a> Iterator for PrefixBatchIterator<'a> {
    type Item = (Vec<&'a str>, bool);

    fn next(&mut self) -> Option<Self::Item> {
        let mut output = Vec::with_capacity(10);
        let start = self.data.len() - self.remainder;
        let mut batch_ip_type = None;
        for prefix in self.data.into_iter().skip(start).map(|p| p.prefix.as_str()) {
            if output.len() >= 20 {
                break;
            }

            let prefix_type = match is_ipv6(prefix) {
                Ok(t) => t,
                Err(e) => {
                    println!("Can't parse IP of prefix {}: {}, skipping", prefix, e);
                    continue;
                }
            };

            if batch_ip_type.is_none() {
                batch_ip_type = Some(prefix_type);
                output.push(prefix);
            } else if Some(prefix_type) == batch_ip_type {
                output.push(prefix);
            } else {
                // Different prefix type, lets stop here.
                break;
            }
        }

        if output.len() == 0 {
            None
        } else {
            self.remainder -= output.len();
            Some((output, batch_ip_type?))
        }
    }
}
