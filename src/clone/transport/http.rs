use std::io;

use reqwest::{blocking::Client, header::USER_AGENT};

use crate::{clone::{packet_line::{packet_line_builder::{UploadPackV2RequestBuilder}, pck_negotiator::UploadPackNegotiator}, refs::RefAdvertisement}, utils::print_utils::print_raw_bytes};


pub fn fetch_refs(url: &str) -> Result<Vec<u8>, std::io::Error> {
    let service_url = format!("{}/info/refs?service=git-upload-pack", url.trim_end_matches('/'));

    let client = Client::new();
    let response = client
        .get(&service_url)
        .header(USER_AGENT, "git/2.42.0")
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

        // Construct POST body
        let url = if base_url.ends_with("git-upload-pack") {
            base_url.to_string()
        } else {
            format!("{}/git-upload-pack", base_url.trim_end_matches('/')).to_string()
        };

        let body = UploadPackV2RequestBuilder::new()
            .want(&ref_adv.head.clone().unwrap_or_default())
            .deepen(10)
            .agent("git/2.42.0")
            .fetch_option("thin-pack")
            .fetch_option("ofs-delta")
            .done()
            .build();
    
        let builder = client
            .post(url)
            .header("Content-Type", "application/x-git-upload-pack-request")
            .header("Accept", "application/x-git-upload-pack-result")
            .header("User-Agent", "git/2.42.0")
            .header("git-protocol", "version=2")
            .body(body.clone()); // You’ll need to clone `body` if you want to inspect it

        // Build the request manually
        let request = builder
            .try_clone()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to clone request builder"))?
            .build()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // Print info
        println!("--- HTTP Request ---");
        println!("{} {}", request.method(), request.url());
        for (key, value) in request.headers() {
            println!("{}: {}", key, value.to_str().unwrap_or("<binary>"));
        }
        println!("Body ({} bytes):", request.body().map_or(0, |_| body.len()));

        
        let res = builder.send().map_err(|e| io::Error::new(io::ErrorKind::Other, format!("request failed: {}", e)))?;
        let res_bytes = &res.bytes().map_err(|e| io::Error::new(io::ErrorKind::Other, format!("failed reading repsonse: {}", e)))?;
        println!("Response size: {} bytes", res_bytes.len());

        Ok(()) 
    }
}

