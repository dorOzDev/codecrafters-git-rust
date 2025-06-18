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
        let cwd = std::env::current_dir()?;
        let rel_path = normalize_git_path(&path, &cwd).unwrap_or_else(|err| panic!("normalize git path failed: {}", err));

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

pub fn normalize_git_path(full_path: &Path, repo_root: &Path) -> Result<String, String> {
    // Step 1: Try to canonicalize the input path
    let abs_path = full_path.canonicalize()
        .map_err(|e| format!("Failed to canonicalize path '{}': {}", full_path.display(), e))?;

    let repo_root = repo_root.canonicalize()
        .map_err(|e| format!("Failed to canonicalize repo root '{}': {}", repo_root.display(), e))?;

    // Step 2: Try to strip the repo root from the full path
    let rel_path = abs_path.strip_prefix(&repo_root)
        .map_err(|e| format!(
            "Failed to strip prefix:\n  full path: '{}'\n  repo root: '{}'\n  error: {}",
            abs_path.display(),
            repo_root.display(),
            e
        ))?;

    // Step 3: Normalize to forward slashes
    let normalized = rel_path
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");

    Ok(normalized)
}