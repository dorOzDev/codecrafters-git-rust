use std::io;

use crate::{hash::{GitHash, HASH_SIZE_BYTES}, objects::{read_object, FileMode, ObjectType}};

pub struct TreeEntry {
    pub mode: FileMode,
    pub object_type: ObjectType,
    pub hash: GitHash,
    pub name: String
}

pub struct Tree {
    pub entries: Vec<TreeEntry>
}

impl Tree {
    pub fn walk_tree<F: FnMut(&TreeEntry, &str)>(&self, parent_path: &str,callback: &mut F, recursive: bool) -> io::Result<()> {
        for entry in &self.entries {
            let full_path = if parent_path.is_empty() {
                entry.name.clone()
            } else {
                format!("{}/{}", parent_path, entry.name)
            };

            callback(entry, &full_path);

            if recursive && entry.object_type == ObjectType::Tree {
                let sub_tree = Tree::load_tree_from_hash(&entry.hash.to_hex())?;
                sub_tree.walk_tree(&full_path, callback, recursive)?;
            }
        }

        Ok(())
    }

    pub fn load_tree_from_hash(hash: &str) -> io::Result<Tree> {
        let (object_type, content) = read_object(hash)?;
        if object_type != ObjectType::Tree {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!(
                "Expected tree object, got: {}", object_type.as_str()
            )));
        }

    let mut cursor = &content[..];
    let mut entries = Vec::new();

    while !cursor.is_empty() {
        let space_index = cursor.iter().position(|&b| b == b' ').ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Malformed tree entry: missing space"))?;
        let mode_str = std::str::from_utf8(&cursor[..space_index]).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8 in mode"))?;
        let mode = FileMode::from_octal_str(mode_str).ok_or_else(|| {io::Error::new(io::ErrorKind::InvalidData, format!("Unknown file mode: {}", mode_str))})?;
        cursor = &cursor[space_index + 1..];

        let null_index = cursor.iter().position(|&b| b == 0).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Malformed tree entry: missing null"))?;
        let name = std::str::from_utf8(&cursor[..null_index])
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8 in name"))?
            .to_string();
        cursor = &cursor[null_index + 1..];

        if cursor.len() < HASH_SIZE_BYTES {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Unexpected end of hash data"));
        }
        let hash_bytes = &cursor[..HASH_SIZE_BYTES];
        let hash = GitHash::from_raw_bytes(hash_bytes);
        cursor = &cursor[HASH_SIZE_BYTES..];
        let object_type = ObjectType::from_mode(&mode);

        entries.push(TreeEntry {
            mode,
            object_type,
            hash,
            name,
        });
    }

    Ok(Tree { entries })
    }
}
