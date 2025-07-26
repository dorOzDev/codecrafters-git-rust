use std::fmt::{Display, Formatter};
use std::io::{self, Read, Write};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use flate2::write::ZlibEncoder;
use flate2::read::ZlibDecoder;
use flate2::Compression;

use crate::constants::{GIT_OBJECTS_DIR};
use crate::hash::GitHash;
use crate::utils::file_utils::read_file;


#[derive(Debug, PartialEq, Eq)]
pub enum ObjectType {
    Blob,
    Tree,
    Commit,
    Unknown(u8),
}

impl ObjectType {
    pub const UNKNOWN_FROM_STR_SENTINEL: u8 = 0xff;

    pub fn as_str(&self) -> &'static str {
        match self {
            ObjectType::Blob => "blob",
            ObjectType::Tree => "tree",
            ObjectType::Commit => "commit",
            ObjectType::Unknown(_) => "unknown",
        }
    }

    pub fn from_str(str: &str) -> Self {
        match str.to_ascii_lowercase().as_str() {
            "blob" => ObjectType::Blob,
            "tree" => ObjectType::Tree,
            "commit" => ObjectType::Commit,
            _ => ObjectType::Unknown(ObjectType::UNKNOWN_FROM_STR_SENTINEL),
        }
    }

    pub fn from_mode(mode: &FileMode) -> Self {
        match mode {
            FileMode::Directory=> ObjectType::Tree,
            _ => ObjectType::Blob,
        }
    }

    pub fn from_code(code: u8) -> Self {
        match code {
            1 => ObjectType::Commit,
            2 => ObjectType::Tree,
            3 => ObjectType::Blob,
            other => ObjectType::Unknown(other),
        }
    }
}

impl Display for ObjectType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Clone)]
pub struct Person {
    pub name: String,
    pub email: String,
    pub timestamp: i64,
    pub timezone: String,
}

impl std::fmt::Display for Person {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} <{}> {} {}", self.name, self.email, self.timestamp, self.timezone)
    }
}

pub fn encode_object(obj_type: ObjectType, data: &[u8]) -> Vec<u8> {
    let header = format!("{} {}\0", obj_type.as_str(), data.len());
    let mut result = Vec::with_capacity(header.len() + data.len());
    result.extend(header.as_bytes());
    result.extend(data);
    result
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FileMode {
    Normal = 0o100644,
    Executable = 0o100755,
    Symlink = 0o120000,
    Directory = 0o040000,
}

impl FileMode {
    pub fn from_octal_str(s: &str) -> Option<Self> {
        // Parse from base-8 string (like "40000", "100644", etc.)
        let mode = u32::from_str_radix(s, 8).ok()?;
        Self::from_u32(mode)
    }

    pub fn from_u32(mode: u32) -> Option<Self> {
        match mode {
            0o100644 => Some(FileMode::Normal),
            0o100755 => Some(FileMode::Executable),
            0o120000 => Some(FileMode::Symlink),
            0o040000 => Some(FileMode::Directory),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            FileMode::Normal => "100644",
            FileMode::Executable => "100755",
            FileMode::Symlink => "120000",
            FileMode::Directory => "40000",
        }
    }
    
    pub fn from_path(path: &Path) -> io::Result<Self> {
        let metadata = fs::symlink_metadata(path)?;

        if metadata.file_type().is_symlink() {
            return Ok(FileMode::Symlink);
        }

        if metadata.is_dir() {
            return Ok(FileMode::Directory);
        }

        if metadata.is_file() {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let perm = metadata.permissions().mode();
                if perm & 0o111 != 0 {
                    return Ok(FileMode::Executable);
                }
            }

            return Ok(FileMode::Normal);
        }

        Err(io::Error::new(io::ErrorKind::InvalidData, "Unsupported file type"))
    }

}


impl Display for FileMode {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub fn read_object(hash_str :&str) -> io::Result<(ObjectType, Vec<u8>)> {
    let hash = GitHash::from_hex(hash_str)?;
    let (dir, file) = hash.to_path_parts();

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
/*
    write the object to the disk and return the calculated hash value of the object
*/
pub fn write_object(object_type: ObjectType, data: &[u8]) -> io::Result<GitHash> {
    let (hash, encoded) = hash_object(object_type, data);
    let (dir, file) = hash.to_path_parts();
    let mut path = PathBuf::from(GIT_OBJECTS_DIR);

    path.push(dir);
    fs::create_dir_all(&path)?;
    path.push(file);

    if !path.exists() {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::fast());
        encoder.write_all(&encoded)?;
        let compressed = encoder.finish()?;
        fs::write(path, compressed)?;
    }

    Ok(hash)
}
/*
    write the object to the disk from path and return the calculated hash value of the object
*/
pub fn write_object_from_path(object_type: ObjectType, file_path: &Path) -> io::Result<GitHash> {
    let contents = read_file(file_path)?;
    write_object(object_type, &contents)
}

pub fn hash_object(object_type: ObjectType, data: &[u8]) -> (GitHash, Vec<u8>) {
    let encoded = encode_object(object_type, data);
    (GitHash::from_bytes(&encoded), encoded)
}