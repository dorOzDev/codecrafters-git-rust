use crate::{hash::GitHash, objects::FileMode};

#[derive(Debug, Clone)]
pub struct IndexEntry {
    pub mode: FileMode,
    pub path: String,
    pub hash: GitHash,
}