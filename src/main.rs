mod commands;
mod objects;
use std::env;
use std::io;
pub mod constants;
pub mod index;
pub mod hash;

fn main() -> io::Result<()> {
    eprintln!("Logs from your program will appear here!");
    let args: Vec<String> = env::args().skip(1).collect(); // skip program name
    commands::run(&args)
}
