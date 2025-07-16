use std::io::Read as StdRead;
use std::io;

/// A generic streamer that reads data in chunks and stops when a callback returns true.
pub struct Streamer<R: StdRead> {
    reader: R,
    chunk_size: usize,
}

impl<R: StdRead> Streamer<R> {
    pub fn new(reader: R, chunk_size: usize) -> Self {
        Self { reader, chunk_size }
    }

    pub fn stream<F>(&mut self, mut process_chunk: Option<F>) -> io::Result<(u64, Option<Vec<u8>>)> 
    where
        F: FnMut(&[u8], u64) -> Option<bool>,
    {
        let mut buf = vec![0u8; self.chunk_size];
        let mut total_read = 0u64;
        let mut last_buf: Option<Vec<u8>> = None;
        loop {
            let n = self.reader.read(&mut buf)?;
            if n == 0 {
                break;
            }
            total_read += n as u64;
            if let Some(ref mut cb) = process_chunk {
                if cb(&buf[..n], total_read) == Some(true) {
                    last_buf = Some(buf[..n].to_vec());
                    break;
                }
            }
        }
        Ok((total_read, last_buf))
    }
}
