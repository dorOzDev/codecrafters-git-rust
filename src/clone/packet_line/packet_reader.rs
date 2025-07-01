use std::io::{self, BufRead};

use git_packetline::{PacketLineRef, StreamingPeekableIter};


pub struct PacketReader<R: BufRead> {
    inner: StreamingPeekableIter<R>,
    bytes_read: usize,
    expected_len: usize,
}

impl<R: BufRead> PacketReader<R> {
    pub fn new(reader: R, expected_len: usize) -> Self {
        Self { 
            inner: StreamingPeekableIter::new(reader, &[]), 
            bytes_read: 0,
            expected_len: expected_len
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

    pub fn validate_consumed(&self) -> io::Result<()> {
        if self.bytes_read == self.expected_len {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!(
                    "Expected to read {} bytes but only read {}",
                    self.expected_len, self.bytes_read
                ),
            ))
        }
    }    
}

pub const GIT_PACKET_LINE_HEADER_LEN: usize = 4;