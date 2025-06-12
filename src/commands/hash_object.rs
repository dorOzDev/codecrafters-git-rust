use std::{fs, io::{self, Read, Write}, path::PathBuf};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::fs::File;
use sha1::{Sha1, Digest};
use crate::objects::{ObjectType, encode_object};

pub fn run(args: &[String]) -> io::Result<()> {
    if args.len() == 3 && args[1] == "-w" {
        match hash_object(&args[2], true) {
        Ok(hash) => {
            println!("{}", hash);
            Ok(())
        }
        Err(e) => Err(io::Error::new(io::ErrorKind::Other, e.to_string())),
    }
    } else {
        eprintln!("unsupported sub command");
        std::process::exit(1)
    }
}

fn hash_object(file_name: &str, write: bool) -> Result<String, Box<dyn std::error::Error>> {
    let file_path = PathBuf::from(&file_name);
    let mut file = File::open(&file_path)?;
    let mut contents  = Vec::new();
    file.read_to_end(&mut contents)?;
    let object = encode_object(ObjectType::Blob, &contents);
    let mut hasher = Sha1::new();
    hasher.update(&object);

    let hash = hex::encode(hasher.finalize());

    if write {
        let(dir, file) = hash.split_at(2);
        let mut path = PathBuf::from(".git/objects");
        path.push(dir);
        fs::create_dir(&path)?;
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