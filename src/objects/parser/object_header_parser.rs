use std::{fmt::Display, io::{self, Read}};

use crate::{objects::ObjectType, utils::streamer::BufferedStreamCursor};

#[derive(Debug)]
pub struct PackObjectHeader {
    pub object_type: ObjectType,
    pub size: u64,
    pub header_size: usize,
}

pub fn parse_object_header<R: Read>(cursor: &mut BufferedStreamCursor<R>) -> io::Result<PackObjectHeader> {
    let first_byte = cursor.read(1)?[0];
    let mut size = (first_byte & 0x0F) as u64;
    let mut shift = 4;
    let mut header_size = 1;

    let object_type = ObjectType::from_code((first_byte >> 4) & 0x07);

    if first_byte & 0x80 != 0 {
        loop {
            let next_byte = cursor.read(1)?[0];
            size |= ((next_byte & 0x7F) as u64) << shift;
            shift += 7;
            header_size += 1;

            if next_byte & 0x80 == 0 {
                break;
            }
        }
    }

    Ok(PackObjectHeader {
        object_type,
        size,
        header_size,
    })
}

impl Display for PackObjectHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PackObjectHeader(type: {}, size: {}, header_size: {})",
            self.object_type, self.size, self.header_size
        )
    }
}