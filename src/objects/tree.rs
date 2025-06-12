use crate::objects::{ObjectType, FileMode};

pub struct TreeEntry {
    pub mode: FileMode,
    pub object_type: ObjectType,
    pub hash: String,
    pub name: String
}

pub struct Tree {
    pub entries: Vec<TreeEntry>
}

impl Tree {
    pub fn walk_tree<F: FnMut(&TreeEntry, &str)>(&self, parent_path: &str, callback: &mut F) {
        for entry in &self.entries {
            let full_path = if parent_path.is_empty() {
                entry.name.clone()
            } else {
                format!("{}/{}", parent_path, entry.name)
            };

            callback(entry, &full_path);

            if entry.object_type == ObjectType::Tree {
                //TODO perform 
                //let subtree = load_tree_from_hash();
                //subtree.walk_tree(&full_path, callback);
            }
        }
    }

    
}