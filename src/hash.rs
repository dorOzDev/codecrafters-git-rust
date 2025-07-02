use core::fmt;
use std::io;

use sha1::{Sha1, Digest};

pub const HASH_SIZE_BYTES: usize = 20;
pub const HASH_HEX_LENGTH: usize = 40;


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GitHash([u8; HASH_SIZE_BYTES]);

impl GitHash {

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut hasher = Sha1::new();
        hasher.update(bytes);
        let result = hasher.finalize();

        let mut hash = [0u8; HASH_SIZE_BYTES];
        hash.copy_from_slice(&result);
        Self(hash)
    }

    pub fn from_hex(s: &str) -> io::Result<Self> {
        if s.len() != HASH_HEX_LENGTH {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("invalid hex length, expected {}, got {}", HASH_HEX_LENGTH, s.len())));
        }
        let bytes = hex::decode(s).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        let mut arr = [0u8; HASH_SIZE_BYTES];
        arr.copy_from_slice(&bytes);
        Ok(Self(arr))
    }

    pub fn from_raw_bytes(bytes: &[u8]) -> Self {
        let mut arr = [0u8; HASH_SIZE_BYTES];
        arr.copy_from_slice(bytes);
        Self(arr)
    }

    pub fn from_raw_str(s: &str) -> io::Result<Self> {
        let bytes = s.as_bytes();

        if bytes.len() != HASH_SIZE_BYTES {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("invalid byes length, expected {}, got {}", HASH_SIZE_BYTES, bytes.len())));
        }

        Ok(Self::from_raw_bytes(bytes))
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
    
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn to_path_parts(&self) -> (String, String) {
        let hex = self.to_hex();
        let (dir, file) = hex.split_at(2);
        (dir.to_string(), file.to_string())
    }

    pub fn hash_version() -> &'static str {
        return "sha1";
    }
}

impl fmt::Display for GitHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hex = self.to_hex();
        let byte_string = self
            .as_bytes()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join(" ");

        write!(
            f,
            "GitHash {{ hex: {}, bytes: [{}] }}",
            hex, byte_string
        )
    }
}