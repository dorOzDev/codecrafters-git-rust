pub mod init;
pub mod cat_file;
pub mod hash_object;

use std::io;

pub fn run(args: &[String]) -> io::Result<()> {
    match args.get(0).map(String::as_str) {
        Some("init") => init::run(),
        Some("cat-file") => cat_file::run(args),
        Some("hash-object") => {
            match hash_object::run(args) {
                Ok(hash) => {
                    println!("{}", hash);
                    Ok(())
                }
                Err(e) => Err(io::Error::new(io::ErrorKind::Other, e.to_string()))
            }
        }
        Some(cmd) => {
            eprintln!("unknown command: {}", cmd);
            Ok(())
        }
        None => {
            eprintln!("Please provide a command.");
            Ok(())
        }
    }
}
