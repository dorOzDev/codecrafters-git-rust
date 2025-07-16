use crate::utils::streamer::Streamer;
use std::{io::{self, Result, Read}, path::Path};
use bytes::Bytes;
use reqwest::blocking::Response;



pub fn unpack_pkt_res(res: Response, repo_root: &Path) -> Result<()> {
    if !res.status().is_success() {
        return Err(io::Error::new(io::ErrorKind::Other, format!("unable to unpack response, response status is: {}", res.status())))
    }
    // Use a chunk size of 8192 (8 KB) for streaming
    let chunk_size = 8192;
    let mut res = res;
    println!("Streaming response to find PACK header...");
    match stream_until_pack(&mut res, chunk_size)? {
        Some(offset) => println!("Found PACK header at offset {}", offset),
        None => println!("PACK header not found in response."),
    }

    Ok(())
}

pub fn stream_until_pack<R: Read>(mut reader: R, chunk_size: usize) -> io::Result<Option<usize>> {
    let mut found_offset: Option<usize> = None;
    let mut window = Vec::new();
    let pack_magic = b"PACK";
    let mut streamer = Streamer::new(&mut reader, chunk_size);
    println!("Data before PACK (utf8):");
    let process = |chunk: &[u8], _so_far: u64| -> Option<bool> {
        let search_start = if window.len() >= 3 { window.len() - 3 } else { 0 };
        window.extend_from_slice(chunk);
        if let Some(pos) = window[search_start..].windows(4).position(|w| w == pack_magic) {
            // Print only the bytes before the PACK magic in the window
            let end = search_start + pos;
            if end > 0 {
                let s = String::from_utf8_lossy(&window[..end]);
                print!("{}", s);
            }
            found_offset = Some(end);
            return Some(true);
        } else {
            // If PACK not found, print all except the last 3 bytes (which may be part of PACK in next chunk)
            if window.len() > 3 {
                let s = String::from_utf8_lossy(&window[..window.len()-3]);
                print!("{}", s);
                // Retain only the last 3 bytes in the window
                window.drain(..window.len()-3);
            }
        }
        None
    };
    let (_total_read, _last_buf) = streamer.stream(Some(process))?;
    if let Some(offset) = found_offset {
        println!();
        Ok(Some(offset))
    } else {
        println!();
        println!("No PACK header found.");
        Ok(None)
    }
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