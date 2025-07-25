use crate::utils::streamer::{BufferedStreamCursor};
use std::{io::{self, Result, Read}, path::Path};
use reqwest::blocking::Response;

pub fn unpack_pkt_res(res: Response, repo_root: &Path) -> Result<()> {
    if !res.status().is_success() {
        return Err(io::Error::new(io::ErrorKind::Other, format!("unable to unpack response, response status is: {}", res.status())))
    }

    let mut cursor = BufferedStreamCursor::with_chunk_size(res, 128);
    print_lines_until_pack(&mut cursor)?;
    let pack_header = parse_pack_header(&mut cursor)?;
    println!("Pack Header numer of objects: {:?}, Version: {}", pack_header.num_objects, pack_header.version);
    cursor.drain_consumed();
    Ok(())
}

pub fn print_lines_until_pack<R: Read>(cursor: &mut BufferedStreamCursor<R>) -> io::Result<()> {
    let pack_magic = b"PACK";

    loop {
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
        let content = cursor.peek(content_len)?;

        if let Some(pos) = content.windows(4).position(|w| w == pack_magic) {
            let before = &content[..pos];
            print!("{}", String::from_utf8_lossy(before));
            cursor.advance(pos);
            break; 
        } else {
            print!("{}", String::from_utf8_lossy(content));
            cursor.advance(content_len);
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

    /// Reads and parses the full 12-byte pack header from the stream cursor.
    pub fn from_cursor<R: Read>(cursor: &mut BufferedStreamCursor<R>) -> io::Result<Self> {
        cursor.ensure_available(Self::SIZE)?;
        let magic = cursor.peek(4)?;
        if magic != b"PACK" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Expected 'PACK' magic at cursor, found {:?}", magic),
            ));
        }

        // Now consume the 4-byte magic
        cursor.advance(4);

        // Read and parse version and num_objects
        let header_bytes = cursor.read(8)?; // version (4) + num_objects (4)
        let version = u32::from_be_bytes(header_bytes[0..4].try_into().unwrap());
        let num_objects = u32::from_be_bytes(header_bytes[4..8].try_into().unwrap());

        Ok(Self {
            signature: *b"PACK",
            version,
            num_objects,
        })
    }
}


pub fn parse_pack_header<R: Read>(cursor: &mut BufferedStreamCursor<R>) -> io::Result<PackHeader> {
    PackHeader::from_cursor(cursor)
}