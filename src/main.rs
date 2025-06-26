mod commands;
mod objects;
use std::env;
use std::io;

pub mod clone;
pub mod constants;
pub mod index;
pub mod hash;
pub mod utils;
fn main() -> io::Result<()> {
    env_logger::init();
    eprintln!("Logs from your program will appear here!");
    let args: Vec<String> = env::args().skip(1).collect(); // skip program name
    commands::run(&args)
}
