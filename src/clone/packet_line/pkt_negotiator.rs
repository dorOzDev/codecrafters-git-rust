use std::{collections::HashMap,  io::{self, Result}, sync::Arc};
use git_url_parse::{GitUrl, Scheme};
use once_cell::sync::Lazy;
use reqwest::blocking::Response;
use crate::{clone::{refs::RefAdvertisement, transport::{http::HttpNegotiator, ssh::SshNegotiator}}};
 

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

pub fn negogiate_want(ref_ads: &RefAdvertisement, url: &str) -> Result<Response> {
    let git_url = GitUrl::parse(&url).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let negotiator = get_negotiator(&git_url.scheme).ok_or_else(|| {io::Error::new(io::ErrorKind::Unsupported, format!("unsupported scheme: {}", git_url.scheme))})?;
    let res = negotiator.negogiate(&url, &ref_ads)?;

    if !res.status().is_success() {
        eprintln!("Fetch failed with status: {}", res.status());
        return Err(io::Error::new(io::ErrorKind::Other, format!("Git server responsed with failure status: {}", res.status())))
    }

    Ok(res)
}