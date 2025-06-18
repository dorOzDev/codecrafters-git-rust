use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

pub fn read_file(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    Ok(contents)
}
