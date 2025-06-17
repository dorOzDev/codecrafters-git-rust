use std::{fs, io::{self, Read, Write}, path::PathBuf};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::fs::File;
use crate::{hash::GitHash, objects::{encode_object, ObjectType}};
use crate::constants::*;

pub fn run(args: &[String]) -> io::Result<()> {
    if args.len() == 3 && args[1] == "-w" {
        let hash = hash_object(&args[2], true)?;
        println!("{}", hash.to_hex());
        Ok(())
    } else {
        eprintln!("unsupported sub command");
        std::process::exit(1);
    }
}

pub fn hash_object(file_name: &str, write_to_fs: bool) -> io::Result<GitHash> {
    let file_path = PathBuf::from(&file_name);
    let mut file = File::open(&file_path)?;
    let mut contents  = Vec::new();
    file.read_to_end(&mut contents)?;
    let object = encode_object(ObjectType::Blob, &contents);
    let hash = GitHash::from_bytes(&object);

    if write_to_fs {
        let(dir, file) = hash.to_path_parts();
        let mut path = PathBuf::from(GIT_OBJECTS_DIR);
        path.push(dir);
        fs::create_dir_all(&path)?;
        path.push(file);

        if !path.exists() {
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&object)?;
            let compressed = encoder.finish()?;
            fs::write(path,compressed)?;
        }
    }
    Ok(hash)
}