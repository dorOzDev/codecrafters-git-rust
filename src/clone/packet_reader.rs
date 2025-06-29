use std::io::{self, BufRead};

use git_packetline::{PacketLineRef, StreamingPeekableIter};


pub struct PacketReader<R: BufRead> {
    inner: StreamingPeekableIter<R>,
    bytes_read: usize,
}

impl<R: BufRead> PacketReader<R> {
    pub fn new(reader: R) -> Self {
        Self { 
            inner: StreamingPeekableIter::new(reader, &[]), 
            bytes_read: 0,
         }   
    }

    pub fn read_line(&mut self) -> io::Result<Option<PacketLineRef<'_>>> {
        match self.inner.read_line() {
            Some(Ok(packet)) => {
                let payload_len = match packet {
                    Ok(PacketLineRef::Data(d)) => d.len() + GIT_PACKET_LINE_HEADER_LEN,
                    _ => GIT_PACKET_LINE_HEADER_LEN,
                };
                self.bytes_read += payload_len;
                Ok(Some(packet.unwrap()))
            },
            Some(Err(err)) => Err(io::Error::new(io::ErrorKind::InvalidData, err)),
            None => Ok(None),
        }
    }

    pub fn total_bytes_read(&self) -> usize {
        self.bytes_read
    }
}

pub const GIT_PACKET_LINE_HEADER_LEN: usize = 4;