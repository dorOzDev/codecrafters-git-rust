use reqwest::{blocking::Client, header::USER_AGENT};


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