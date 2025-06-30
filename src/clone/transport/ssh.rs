use crate::clone::transport::pck_negotiator::UploadPackNegotiator;


pub struct SshNegotiator;

impl UploadPackNegotiator for SshNegotiator {

    fn negogiate(&self, url: &str, ref_advertied: &crate::clone::refs::RefAdvertisement) -> std::io::Result<()> {
        todo!()
    }
}