use reqwest::blocking::Response;

use crate::clone::{packet_line::pkt_negotiator::UploadPackNegotiator, refs::RefAdvertisement};



pub struct SshNegotiator;

impl UploadPackNegotiator for SshNegotiator {

    fn negogiate(&self, _url: &str, _ref_advertied: &RefAdvertisement) -> std::io::Result<Response> {
        todo!()
    }
}