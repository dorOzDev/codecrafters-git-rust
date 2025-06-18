pub mod init;
pub mod cat_file;
pub mod hash_object;
pub mod ls_tree;
pub mod add;
pub mod write_tree;
use std::io;

pub fn run(args: &[String]) -> io::Result<()> {
    match args.get(0).map(String::as_str) {
        Some("init") => init::run(),
        Some("cat-file") => cat_file::run(args),
        Some("hash-object") => hash_object::run(args),
        Some("ls-tree") => ls_tree::run(&args[1..]),
        Some("add") => add::run(&args[1..]),
        Some("write-tree") => write_tree::run(),
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
