use std::{collections::HashSet, io, path::Path};

use crate::{commands::add, constants::INDEX_PATH, hash::GitHash, index::{index::read_index, index_entry::IndexEntry}, objects::{write_object, FileMode, ObjectType}};



pub fn run() -> io::Result<()> {
    // TODO delete this code once done with the codecrafters challenge.
    // we have shifted away from their challenge and can do a proper write-tree reading from the staging.
    add::run(&vec![".".to_string()])?;

    let entries = match read_index(Path::new(INDEX_PATH)) {
        Ok(vec) => vec,
        Err(_) => vec![],
    };

    if entries.is_empty() {
        let hash = write_object(ObjectType::Tree, b"")?;
        println!("{}", hash.to_hex());
        return Ok(());
    }

    let hash = build_tree(&entries, "")?;
    println!("{}", hash.to_hex());

    Ok(())
}

fn build_tree(entries: &[IndexEntry], prefix: &str) -> io::Result<GitHash> {
    let mut seen_dirs = HashSet::new();
    let mut result_entries: Vec<(FileMode, String, GitHash)> = Vec::new();

    for entry in entries {
        if !entry.path.starts_with(prefix) {
            continue;
        }

        let rest = &entry.path[prefix.len()..];

        if let Some(pos) = rest.find('/') {
            let dirname = &rest[..pos];
            if seen_dirs.insert(dirname.to_string()) {
                let full_prefix = format!("{}{}/", prefix, dirname);
                let sub_entries: Vec<IndexEntry> = entries
                    .iter()
                    .filter(|e| e.path.starts_with(&full_prefix))
                    .cloned()
                    .collect();

                let subtree_hash = build_tree(&sub_entries, &full_prefix)?;
                result_entries.push((FileMode::Directory, dirname.to_string(), subtree_hash));
            }
        } else {
            result_entries.push((
                entry.mode.clone(),
                rest.to_string(),
                entry.hash.clone(),
            ));
        }
    }

    // Sort all entries by name (binary order as Git requires)
    result_entries.sort_by(|a, b| a.1.as_bytes().cmp(b.1.as_bytes()));

    // Serialize tree content
    let mut content = Vec::new();
    for (mode, name, hash) in result_entries {
        let mut line = format!("{} {}", mode.as_str(), name).into_bytes();
        line.push(0);
        line.extend_from_slice(&hash.as_bytes());
        content.extend(line);
    }

    write_object(ObjectType::Tree, &content)
}
