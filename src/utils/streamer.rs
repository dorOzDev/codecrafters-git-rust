use std::io::{Read, Write};
use std::io;

use sha1::Digest;

pub struct BufferedStreamCursor<R: Read> {
    reader: R,
    buffer: Vec<u8>,
    cursor: usize,
    eof: bool,
    chunk_size: usize,
}

impl<R: Read> BufferedStreamCursor<R> {
    pub fn with_chunk_size(reader: R, chunk_size: usize) -> Self {
        Self {
            reader,
            buffer: Vec::new(),
            cursor: 0,
            eof: false,
            chunk_size,
        }
    }

    pub fn ensure_available(&mut self, n: usize) -> io::Result<()> { 
        while self.available() < n && !self.eof {
            let mut temp = vec![0; self.chunk_size];
            let n_read = self.reader.read(&mut temp)?;
            if n_read == 0 {
                self.eof = true;
                break;
            }
            self.buffer.extend_from_slice(&temp[..n_read]);
        }
        Ok(())
    }

    pub fn available(&self) -> usize {
        self.buffer.len().saturating_sub(self.cursor)
    }

    pub fn read(&mut self, n: usize) -> io::Result<&[u8]> {
        self.ensure_available(n)?;
        let slice = &self.buffer[self.cursor..self.cursor + n];
        self.cursor += n;
        Ok(slice)
    }

    pub fn search(&self, pattern: &[u8]) -> Option<usize> {
        self.buffer[self.cursor..].windows(pattern.len()).position(|w| w == pattern)
    }

    pub fn advance(&mut self, n: usize) {
        self.cursor += n;
    }

    pub fn position(&self) -> usize {
        self.cursor
    }

    pub fn take(&mut self, n: usize) -> io::Result<Vec<u8>> {
        let bytes = self.read(n)?;
        Ok(bytes.to_vec())
    }

    pub fn peek(&mut self, n: usize) -> io::Result<&[u8]> {
        self.ensure_available(n)?;
        Ok(&self.buffer[self.cursor..self.cursor + n])
    }

    /// Drops all bytes that have already been read
    pub fn drain_consumed(&mut self) {
        if self.cursor > 0 {
            self.buffer.drain(0..self.cursor);
            self.cursor = 0;
        }
    }    
}

pub struct TeeWriter<'a, W: Write, H: Digest> {
    writer: &'a mut W,
    hasher: &'a mut H,
}

impl<'a, W: Write, H: Digest> TeeWriter<'a, W, H> {
    pub fn new(writer: &'a mut W, hasher: &'a mut H) -> Self {
        Self { writer, hasher }
    }

    pub fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.writer.write_all(buf)?;
        self.hasher.update(buf);
        Ok(())
    }
}
