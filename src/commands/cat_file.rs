use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;
use flate2::read::ZlibDecoder;

pub fn run(args: &[String]) -> io::Result<()> {
    if args.len() == 3 && args[1] == "-p" {
        let hash = &args[2];
        return cat_file_print(hash);
    } else {
        eprintln!("Usage: git-rust cat-file -p <hash>");
        std::process::exit(1);
    }
}

fn cat_file_print(hash: &str) -> io::Result<()> {
    if hash.len() != 40 {
        eprintln!("Invalid hash length: expected 40 characters.");
        std::process::exit(1);
    }

    let (dir, file) = hash.split_at(2);
    let mut object_path = PathBuf::from(".git/objects");
    object_path.push(dir);
    object_path.push(file);

    let file = File::open(&object_path)?;
    let mut decoder = ZlibDecoder::new(file);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;

    if let Some(null_index) = decompressed.iter().position(|&b| b == 0) {
        let header = &decompressed[..null_index];
        let content = &decompressed[null_index + 1..];

        let header_str = String::from_utf8_lossy(header);
        let mut parts = header_str.split(' ');
        let object_type = parts.next().unwrap_or("");

        match object_type {
            "blob" => {
                print!("{}", String::from_utf8_lossy(content));
            }
            _ => {
                eprintln!("Unsupported object type: {}", object_type);
            }
        }
    } else {
        eprintln!("Malformed object: missing header");
    }

    Ok(())
}
