use crate::utils::streamer::{BufferedStreamCursor};
use std::{io::{self, Result, Read}, path::Path};
use reqwest::blocking::Response;

pub fn unpack_pkt_res(res: Response, repo_root: &Path) -> Result<()> {
    if !res.status().is_success() {
        return Err(io::Error::new(io::ErrorKind::Other, format!("unable to unpack response, response status is: {}", res.status())))
    }

    let mut cursor = BufferedStreamCursor::with_chunk_size(res, 128);
    print_lines_until_pack(&mut cursor)?;

    Ok(())
}
pub fn print_lines_until_pack<R: Read>(cursor: &mut BufferedStreamCursor<R>) -> io::Result<()> {
    let pack_magic = b"PACK";

    loop {
        cursor.ensure_available(4)?;
        let len_bytes = cursor.read(4)?;
        if len_bytes == b"0000" {
            continue; 
        }

        let hex_str = std::str::from_utf8(len_bytes)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid hex in pkt-line"))?;
        let total_len = usize::from_str_radix(hex_str, 16)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid hex value"))?;

        if total_len < 4 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid pkt-line length"));
        }

        let content_len = total_len - 4;
        cursor.ensure_available(content_len)?;
        let content = cursor.read(content_len)?;

        if let Some(pos) = content.windows(4).position(|w| w == pack_magic) {
            let before = &content[..pos];
            print!("{}", String::from_utf8_lossy(before));
            break; 
        } else {
            print!("{}", String::from_utf8_lossy(content));
        }
    }

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

pub fn find_and_parse_pack_header(bytes: &[u8]) -> io::Result<(usize, PackHeader)> {
    if let Some(offset) = bytes.windows(4).position(|w| w == b"PACK") {
        let header_start = offset;
        let header_end = header_start + PackHeader::SIZE;
        if header_end > bytes.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Not enough bytes for packfile header"));
        }
        let header = PackHeader::from_bytes(&bytes[header_start..header_end])?;
        Ok((header_start, header))
    } else {
        Err(io::Error::new(io::ErrorKind::InvalidData, "PACK magic number not found"))
    }
}