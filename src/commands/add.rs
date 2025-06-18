use std::{fs, io, path::{Path, PathBuf}};

use crate::{constants::{GIT_DIR, INDEX_PATH}, index::{index::{read_index, write_index}, index_entry::IndexEntry}, objects::{write_object_from_path, FileMode}};


pub fn run(args: &[String]) -> io::Result<()> {
    let index_path = Path::new(INDEX_PATH);
    let mut entries = if index_path.exists() {
        match read_index(index_path) {
            Ok(data) => data,
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof || e.kind() == io::ErrorKind::InvalidData => Vec::new(),
            Err(e) => return Err(e),
        }
    } else {
        Vec::new()
    };

    let target_paths = if args.is_empty() || (args.len() == 1 && args[0] == ".") {
        collect_all_files(".")?
    } else {
        args.iter().map(PathBuf::from).collect()
    };

    for path in target_paths {
        if path.starts_with(GIT_DIR) || !path.is_file() {
            continue;
        }

        let hash = write_object_from_path(crate::objects::ObjectType::Blob, &path)?; 

        let mode = FileMode::from_path(&path)?;
        let rel_path = path.strip_prefix(std::env::current_dir()?.as_path()).unwrap_or(&path).to_string_lossy().to_string();

        let entry = IndexEntry {
            mode,
            path: rel_path,
            hash,
        };

        // Replace existing entry or insert new one
        if let Some(existing) = entries.iter_mut().find(|e| e.path == entry.path) {
            *existing = entry;
        } else {
            entries.push(entry);
        }
    }

    write_index(index_path, &entries)
}

fn collect_all_files(root: &str) -> io::Result<Vec<PathBuf>> {
    let mut result = Vec::new();
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() {
            if !path.ends_with(GIT_DIR) {
                result.extend(collect_all_files(&path.to_string_lossy())?);
            }
        } else {
            result.push(path);
        }
    }

    Ok(result)
}