use std::fs;
use std::io;
use crate::constants::*;

pub fn run() -> io::Result<()> {
    fs::create_dir_all(GIT_DIR)?;
    fs::create_dir_all(GIT_OBJECTS_DIR)?;
    fs::create_dir_all(GIT_REF_DIR)?;
    fs::write(GIT_HEAD_PATH, "ref: refs/heads/main\n")?;
    println!("Initialized git directory");
    Ok(())
}
