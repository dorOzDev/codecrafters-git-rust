use std::io;

use reqwest::{blocking::Client, header::USER_AGENT};

use crate::{clone::{packet_line::{packet_line_builder::{UploadPackV2RequestBuilder}, pkt_negotiator::UploadPackNegotiator}, refs::RefAdvertisement}};


pub fn fetch_refs(url: &str) -> Result<Vec<u8>, std::io::Error> {
    let service_url = format!("{}/info/refs?service=git-upload-pack", url.trim_end_matches('/'));

    let client = Client::new();
    let response = client
        .get(&service_url)
        .header(USER_AGENT, GIT_AGENT)
        .send()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    if !response.status().is_success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Request failed with status {}", response.status()),
        ));
    }

    let bytes = response
        .bytes()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    Ok(bytes.to_vec())
}

pub struct HttpNegotiator;

impl UploadPackNegotiator for HttpNegotiator {
    fn negogiate(&self, base_url: &str, ref_adv: &RefAdvertisement) -> std::io::Result<()> {
        let client = Client::new();
        let url = clean_url(&base_url);
        let head_hash = ref_adv.head.as_ref().ok_or_else(|| {io::Error::new(io::ErrorKind::InvalidData, "No HEAD advertised in refs")})?;

        let body = UploadPackV2RequestBuilder::new()
            .want(&head_hash)
            .deepen(10)
            .agent(GIT_AGENT)
            .fetch_option("thin-pack")
            .fetch_option("ofs-delta")
            .done()
            .build();
    
        let res = client
            .post(url)
            .header("Content-Type", "application/x-git-upload-pack-request")
            .header("Accept", "application/x-git-upload-pack-result")
            .header("User-Agent", GIT_AGENT)
            .header("git-protocol", "version=2")
            .body(body.clone())
            .send()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("request failed: {}", e)))?; // You’ll need to clone `body` if you want to inspect it

        let res_bytes = &res.bytes().map_err(|e| io::Error::new(io::ErrorKind::Other, format!("failed reading repsonse: {}", e)))?;
        println!("Response size: {} bytes", res_bytes.len());

        Ok(()) 
    }
}

fn clean_url(url: &str) -> String {

    return if url.ends_with(GIT_UPLOAD_PACK) {
        url.to_string()
    } 
    else {
        format!("{}/{}", url.trim_end_matches('/'), GIT_UPLOAD_PACK).to_string()};
    }
const GIT_UPLOAD_PACK: &'static str = "git-upload-pack";
pub const GIT_AGENT: &'static str = "git/2.42.0";