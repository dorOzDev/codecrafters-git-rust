use std::io::{self};
use crate::objects::{read_object, ObjectType};

pub fn run(args: &[String]) -> io::Result<()> {
    if args.len() == 3 && args[1] == "-p" {
        let hash = &args[2];
        return cat_file_print(hash);
    } else {
        eprintln!("Usage: git-rust cat-file -p <hash>");
        std::process::exit(1);
    }
}

pub fn cat_file_print(hash: &str) -> io::Result<()> {
    let (object_type, content) = read_object(hash)?;

    match object_type {
        ObjectType::Blob => {
            print!("{}", String::from_utf8_lossy(&content));
        }
        _ => {
            eprintln!("Unsupported object type: {}", object_type.as_str());
        }
    }

    Ok(())
}