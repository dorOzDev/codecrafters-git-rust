use std::{collections::BTreeMap, io, path::Path};

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
    let mut content = Vec::new();
    let mut subdirs: BTreeMap<String, Vec<IndexEntry>> = BTreeMap::new();
    let mut file_entries = Vec::new();

    for entry in entries {
        if !entry.path.starts_with(prefix) {
            continue;
        }

        let rest = &entry.path[prefix.len()..];

        if let Some(pos) = rest.find('/') {
            let dirname = &rest[..pos];
            subdirs.entry(dirname.to_string())
                .or_default()
                .push(entry.clone());
        } else {
            file_entries.push((rest.to_string(), entry));
        }
    }

    file_entries.sort_by(|a, b| a.0.cmp(&b.0));

    for (name, entry) in file_entries {
        let mut line = format!("{} {}", entry.mode, name).into_bytes();
        line.push(0);
        line.extend_from_slice(&entry.hash.as_bytes());
        content.extend(line);
    }

    for (dirname, sub_entries) in subdirs {
        let dir_prefix = format!("{}{}/", prefix, dirname);
        let subtree_hash = build_tree(&sub_entries, &dir_prefix)?;
        let mut line = format!("{} {}", FileMode::Directory.as_str(), dirname).into_bytes();
        line.push(0);
        line.extend_from_slice(&subtree_hash.as_bytes());
        content.extend(line);
    }

    write_object(ObjectType::Tree, &content)
}