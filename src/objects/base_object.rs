use std::fmt::{Formatter, Display};


#[derive(Debug, PartialEq, Eq)]
pub enum ObjectType {
    Blob,
    Tree,
}

impl ObjectType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ObjectType::Blob => "blob",
            ObjectType::Tree => "tree",
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