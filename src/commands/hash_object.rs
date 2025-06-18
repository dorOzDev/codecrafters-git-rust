use std::{io, path::Path};
use crate::{objects::{write_object_from_path, ObjectType}};

pub fn run(args: &[String]) -> io::Result<()> {
    if args.len() == 3 && args[1] == "-w" {
        let hash = write_object_from_path(ObjectType::Blob, Path::new(&args[2]))?;
        println!("{}", hash.to_hex());
        Ok(())
    } else {
        eprintln!("unsupported sub command");
        std::process::exit(1);
    }
}