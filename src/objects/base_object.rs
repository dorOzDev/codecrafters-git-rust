use std::fmt::{Formatter, Display};
use std::io::{self, Read};
use std::fs::File;
use std::path::PathBuf;
use flate2::read::ZlibDecoder;

use crate::constants::{GIT_OBJECTS_DIR, HASH_LENGTH};


#[derive(Debug, PartialEq, Eq)]
pub enum ObjectType {
    Blob,
    Tree,
    Unknown,
}

impl ObjectType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ObjectType::Blob => "blob",
            ObjectType::Tree => "tree",
            ObjectType::Unknown => "unknown",
        }
    }

    pub fn from_str(str: &str) -> Self {
        match str.to_ascii_lowercase().as_str() {
            "blob" => ObjectType::Blob,
            "tree" => ObjectType::Tree,
            _ => ObjectType::Unknown,
        }
    }

    pub fn from_mode(mode: &FileMode) -> Self {
        match mode {
            FileMode::Directory=> ObjectType::Tree,
            _ => ObjectType::Blob,
        }
    }
}

pub fn encode_object(obj_type: ObjectType, data: &[u8]) -> Vec<u8> {
    let header = format!("{} {}\0", obj_type.as_str(), data.len());
    let mut result = Vec::with_capacity(header.len() + data.len());
    result.extend(header.as_bytes());
    result.extend(data);
    result
}

#[derive(Debug, PartialEq, Eq)]
pub enum FileMode {
    Normal = 0o100644,
    Executable = 0o100755,
    Symlink = 0o120000,
    Directory = 0o040000,
}

impl FileMode {

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
           "100644" => Some(FileMode::Normal),
            "100755" => Some(FileMode::Executable),
            "120000" => Some(FileMode::Symlink),
            "040000" => Some(FileMode::Directory),
           _ => None
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            FileMode::Normal => "100644",
            FileMode::Executable => "100755",
            FileMode::Symlink => "120000",
            FileMode::Directory => "040000",
        }
    }
}

impl Display for FileMode {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub fn read_object(hash :&str) -> io::Result<(ObjectType, Vec<u8>)> {
    if hash.len() != HASH_LENGTH {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid hash length"));
    }
    
    let (dir, file) = hash.split_at(2);
    let mut object_path = PathBuf::from(GIT_OBJECTS_DIR);
    object_path.push(dir);
    object_path.push(file);

    let file = File::open(&object_path)?;
    let mut decoder = ZlibDecoder::new(file);
    let mut decompressed = Vec::new();
    _= decoder.read_to_end(&mut decompressed);

    if let Some(null_index) = decompressed.iter().position(|&b| b == 0) {
        let header = &decompressed[..null_index];
        let content = decompressed[null_index + 1..].to_vec();
        let header_str = String::from_utf8_lossy(header);
        let object_type_str = header_str.split(' ').next().unwrap_or("");
        let object_type = ObjectType::from_str(object_type_str);
        Ok((object_type, content))
    } else {
        Err(io::Error::new(io::ErrorKind::InvalidData, "Malformed object: missing header"))
    }
}