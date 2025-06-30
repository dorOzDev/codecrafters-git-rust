use std::{collections::HashMap, io::{self, Result}, sync::Arc};

use git_url_parse::{GitUrl, Scheme};
use once_cell::sync::Lazy;


use crate::clone::{refs::RefAdvertisement, transport::{http::HttpNegotiator, ssh::SshNegotiator}};
 

static NEGOTIATOR_MAP: Lazy<HashMap<&'static str, Arc<dyn UploadPackNegotiator>>> = Lazy::new(|| {
    let mut map: HashMap<&'static str, Arc<dyn UploadPackNegotiator>> = HashMap::new();
    map.insert("http", Arc::new(HttpNegotiator));
    map.insert("https", Arc::new(HttpNegotiator));
    map.insert("ssh", Arc::new(SshNegotiator));
    map
});

fn get_negotiator(scheme: &Scheme) -> Option<Arc<dyn UploadPackNegotiator>> {
    let key = scheme.to_string().to_ascii_lowercase(); // "Http" -> "http"
    NEGOTIATOR_MAP.get(key.as_str()).cloned()
}

pub trait UploadPackNegotiator: Send + Sync {
    fn negogiate(&self, url: &str, ref_advertied: &RefAdvertisement) -> Result<()>;
}

pub fn run_upload_pck(ref_ads: &RefAdvertisement, url: &str) -> Result<()> {
    let git_url = GitUrl::parse(&url).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let negotiator = get_negotiator(&git_url.scheme).ok_or_else(|| {io::Error::new(io::ErrorKind::Unsupported, format!("unsupported scheme: {}", git_url.scheme))})?;
    negotiator.negogiate(&url, &ref_ads)?;
    Ok(())
}
