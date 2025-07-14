use std::{fs, io::{self, Result}, path::Path};
use bytes::Bytes;
use reqwest::blocking::Response;



pub fn unpack_pkt_res(res: Response, repo_root: &Path) -> Result<()> {
    if !res.status().is_success() {
        return Err(io::Error::new(io::ErrorKind::Other, format!("unable to unpack response, response status is: {}", res.status())))
    }

    let bytes = res.bytes().map_err(|e| {io::Error::new(io::ErrorKind::Other, format!("failed to read response: {}", e))})?;
    let pack_start = get_pack_start(&bytes);
    if pack_start == bytes.len() {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "PACK section not found in server response"));
    }
    save_packfile(&bytes, pack_start, repo_root)?;


    Ok(())
}

fn save_packfile(bytes: &[u8], pack_start: usize, repo_root: &Path) -> io::Result<()> {
    let pack_data = &bytes[pack_start..];

    let header = PackHeader::from_bytes(pack_data)?;
    println!(
        "Packfile header OK: version={}, objects={}",
        header.version, header.num_objects
    );
    println!("{}", pack_start); 
    let pack_dir = repo_root.join(".git/objects/pack");
    fs::create_dir_all(&pack_dir)?;

    let pack_path = pack_dir.join("clone.pack");
    fs::write(&pack_path, pack_data)?;

    println!("Saved packfile to {}", pack_path.display());
    Ok(())
}
pub struct PackHeader {
    pub signature: [u8; 4],
    pub version: u32,       
    pub num_objects: u32,
}

impl PackHeader {
    pub const SIZE: usize = 12;

    pub fn header_len() -> usize {
        Self::SIZE
    } 

    /// Try to parse a packfile header from the given bytes
    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() < Self::SIZE {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!(
                    "Packfile header too small: expected at least {} bytes, got {}",
                    Self::SIZE,
                    bytes.len()
                ),
            ));
        }

        let signature = <[u8; 4]>::try_from(&bytes[0..4]).unwrap();
        if &signature != b"PACK" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid packfile signature: {:?}", signature),
            ));
        }

        let version = u32::from_be_bytes(bytes[4..8].try_into().unwrap());
        let num_objects = u32::from_be_bytes(bytes[8..12].try_into().unwrap());

        Ok(Self {
            signature,
            version,
            num_objects,
        })
    }
}


fn get_pack_start(bytes: &Bytes) -> usize {
    bytes
    .windows(4)
    .position(|w| w == b"PACK")
    .unwrap_or(bytes.len())
}