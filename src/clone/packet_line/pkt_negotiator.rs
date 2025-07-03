use core::fmt;
use std::{collections::HashMap, io::{self, ErrorKind, Result}, sync::Arc};

use git_url_parse::{GitUrl, Scheme};
use once_cell::sync::Lazy;
use reqwest::blocking::Response;


use crate::{clone::{refs::RefAdvertisement, transport::{http::HttpNegotiator, ssh::SshNegotiator}}, hash::{GitHash, HASH_HEX_LENGTH}};
 

static NEGOTIATOR_MAP: Lazy<HashMap<&'static str, Arc<dyn UploadPackNegotiator>>> = Lazy::new(|| {
    let mut map: HashMap<&'static str, Arc<dyn UploadPackNegotiator>> = HashMap::new();
    map.insert("http", Arc::new(HttpNegotiator));
    map.insert("https", Arc::new(HttpNegotiator));
    map.insert("ssh", Arc::new(SshNegotiator));
    map
});

fn get_negotiator(scheme: &Scheme) -> Option<Arc<dyn UploadPackNegotiator>> {
    let key = scheme.to_string().to_ascii_lowercase();
    NEGOTIATOR_MAP.get(key.as_str()).cloned()
}

pub trait UploadPackNegotiator: Send + Sync {
    fn negogiate(&self, url: &str, ref_advertied: &RefAdvertisement) -> Result<Response>;
}

pub fn run_upload_pck(ref_ads: &RefAdvertisement, url: &str) -> Result<()> {
    let git_url = GitUrl::parse(&url).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let negotiator = get_negotiator(&git_url.scheme).ok_or_else(|| {io::Error::new(io::ErrorKind::Unsupported, format!("unsupported scheme: {}", git_url.scheme))})?;
    let res = negotiator.negogiate(&url, &ref_ads)?;

    if !res.status().is_success() {
        eprintln!("Fetch failed with status: {}", res.status());
    }

    let bytes = res.bytes().map_err(|e| {std::io::Error::new(ErrorKind::Other, format!("failed to read response: {}", e))})?;
    let parsed_upload_pck_vec = parse_fetch_response(&bytes)?;
    for upload_pck in parsed_upload_pck_vec {
        println!("{upload_pck}");
    }
    Ok(())
}

pub fn parse_fetch_response(data: &[u8]) -> std::io::Result<Vec<GitServerRef>> {
    if data.is_empty() {
        println!("no data available to parse");
        return Ok(Vec::new());
    }
    let body_str = std::str::from_utf8(data).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Invalid UTF-8: {}", e)))?;
    let mut refs = Vec::new();

    for line in body_str.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed == "0000" {
            continue;
        }

        let mut parts = trimmed.split_whitespace();

        let hash = match parts.next() {
            Some(h) => {
                if h.len() != HASH_HEX_LENGTH {
                    return Err(io::Error::new(io::ErrorKind::InvalidData,format!("Unsupported object format (hash length {}): expected {}", h.len(), GitHash::hash_version()),))
                }
                h.to_string()
            },
            None => continue,
        };

        let refname = match parts.next() {
            Some(r) => r.to_string(),
            None => continue,
        };

        let mut symref_target = None;
        let mut peeled = None;

        for part in parts {
            if let Some(target) = part.strip_prefix("symref-target:") {
                symref_target = Some(target.to_string());
            } else if let Some(peeled_hash) = part.strip_prefix("peeled:") {
                peeled = Some(peeled_hash.to_string());
            }
        }

        refs.push(GitServerRef {
            hash,
            refname,
            symref_target,
            peeled,
        });
    }

    Ok(refs)
}

pub struct GitServerRef {
    pub hash: String,
    pub refname: String,
    pub symref_target: Option<String>,
    pub peeled: Option<String>,
}

pub struct GitUploadPackResponse {
    pub refs: Vec<GitServerRef>,
    pub capabilities: Vec<String>,
}

impl fmt::Display for GitServerRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} {}", self.hash, self.refname)?;

        if let Some(symref) = &self.symref_target {
            writeln!(f, "  symref-target: {}", symref)?;
        }

        if let Some(peeled) = &self.peeled {
            writeln!(f, "  peeled: {}", peeled)?;
        }

        Ok(())
    }
}
