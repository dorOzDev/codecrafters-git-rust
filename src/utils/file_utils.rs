use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use rand::distr::Alphanumeric;
use rand::Rng;

use bytes::Bytes;

pub fn generate_temp_filename(prefix: Option<&str>) -> OsString {
    let rand_string: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(12)
        .map(char::from)
        .collect();

    let filename = match prefix {
        Some(p) => format!("{}{}.temp", p, rand_string),
        None => format!("{}.temp", rand_string),
    };

    OsString::from(filename)
}


pub fn read_file(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    Ok(contents)
}

pub fn dump_bytes_as_hex(bytes: &Bytes, file_path: &str) -> io::Result<()> {
    let mut file = File::create(file_path)?;

    for (i, chunk) in bytes.chunks(16).enumerate() {
        // Offset at the start of the line
        write!(file, "{:08x}: ", i * 16)?;

        // Hex section
        for byte in chunk.iter() {
            write!(file, "{:02x} ", byte)?;
        }

        // Pad if chunk < 16
        for _ in 0..(16 - chunk.len()) {
            write!(file, "   ")?;
        }

        // ASCII representation
        write!(file, " |")?;
        for byte in chunk.iter() {
            let c = *byte as char;
            write!(
                file,
                "{}",
                if c.is_ascii_graphic() || c == ' ' { c } else { '.' }
            )?;
        }
        writeln!(file, "|")?;
    }

    Ok(())
}