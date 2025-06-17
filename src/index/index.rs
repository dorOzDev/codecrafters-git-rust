use std::{fs::File, io::{self, Read, Write}, path::Path};
use crate::{hash::{GitHash, HASH_SIZE_BYTES}, index::index_entry::IndexEntry, objects::FileMode};

/// High-level descriptor of the index file format (version 1).
/// Encapsulates constants and behavior for parsing header and entries.
#[derive(Debug, Clone, Copy)]
pub struct IndexFormatDescriptor {
    pub magic: &'static [u8],
    pub version: u32,
    pub mode_size: usize,
    pub path_len_size: usize,
    pub hash_size: usize,
    pub max_path_len: usize,
}

impl IndexFormatDescriptor {
    pub const HEADER_SIZE: usize = 3 + 4 + 4; // magic + version + entry count

    pub fn read_header<R: Read>(&self, reader: &mut R) -> io::Result<IndexHeader> {
        let mut magic_buf = vec![0u8; self.magic.len()];
        reader.read_exact(&mut magic_buf)?;
        if magic_buf != self.magic {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid magic"));
        }

        let mut version_buf = [0u8; 4];
        reader.read_exact(&mut version_buf)?;
        let version = u32::from_be_bytes(version_buf);

        if version != self.version {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Unsupported version"));
        }

        let mut count_buf = [0u8; 4];
        reader.read_exact(&mut count_buf)?;
        let entry_count = u32::from_be_bytes(count_buf);

        Ok(IndexHeader { version, entry_count })
    }

    pub fn write_header<W: Write>(&self, writer: &mut W, entry_count: u32) -> io::Result<()> {
        writer.write_all(self.magic)?;
        writer.write_all(&self.version.to_be_bytes())?;
        writer.write_all(&entry_count.to_be_bytes())?;
        Ok(())
    }

    pub fn read_entry<R: Read>(&self, reader: &mut R) -> io::Result<IndexEntry> {
        let mut mode_buf = [0u8; 4];
        reader.read_exact(&mut mode_buf)?;
        let mode_val = u32::from_be_bytes(mode_buf);
        let mode = FileMode::from_u32(mode_val)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, format!("Invalid file mode: {mode_val:#o}")))?;

        let mut len_buf = [0u8; 2];
        reader.read_exact(&mut len_buf)?;
        let path_len = u16::from_be_bytes(len_buf) as usize;
        if path_len > self.max_path_len {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Path too long"));
        }

        let mut path_buf = vec![0u8; path_len];
        reader.read_exact(&mut path_buf)?;
        let path = String::from_utf8(path_buf)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8 in path"))?;

        let mut hash_buf = vec![0u8; self.hash_size];
        reader.read_exact(&mut hash_buf)?;
        let hash = GitHash::from_bytes(&hash_buf);

        Ok(IndexEntry { mode, path, hash })
    }

    pub fn write_entry<W: Write>(&self, writer: &mut W, entry: &IndexEntry) -> io::Result<()> {
        let mode = entry.mode.clone() as u32;
        writer.write_all(&mode.to_be_bytes())?;

        let path_bytes = entry.path.as_bytes();
        let path_len = path_bytes.len();
        if path_len > self.max_path_len {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Path too long"));
        }
        writer.write_all(&(path_len as u16).to_be_bytes())?;
        writer.write_all(path_bytes)?;

        let hash_bytes = entry.hash.as_bytes();
        if hash_bytes.len() != self.hash_size {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid hash size"));
        }
        writer.write_all(hash_bytes)?;

        Ok(())
    }
}

/// Represents the parsed index file header (output of reading the descriptor)
#[derive(Debug)]
pub struct IndexHeader {
    pub version: u32,
    pub entry_count: u32,
}

/// Default descriptor for our index format v1
pub const INDEX_FORMAT_V1: IndexFormatDescriptor = IndexFormatDescriptor {
    magic: b"IDX",
    version: 1,
    mode_size: 4,
    path_len_size: 2,
    hash_size: HASH_SIZE_BYTES,
    max_path_len: u16::MAX as usize,
};

pub fn read_index(path: &Path) -> io::Result<Vec<IndexEntry>> {
    let mut file = File::open(path)?;

    // Use the descriptor to read the header
    let header = INDEX_FORMAT_V1.read_header(&mut file)?;

    let mut entries = Vec::with_capacity(header.entry_count as usize);
    for _ in 0..header.entry_count {
        let entry = INDEX_FORMAT_V1.read_entry(&mut file)?;
        entries.push(entry);
    }

    Ok(entries)
}

pub fn write_index(path: &Path, entries: &[IndexEntry]) -> io::Result<()> {
    let mut file = File::create(path)?;

    // Use the descriptor to write the header
    INDEX_FORMAT_V1.write_header(&mut file, entries.len() as u32)?;

    for entry in entries {
        INDEX_FORMAT_V1.write_entry(&mut file, entry)?;
    }

    Ok(())
}
