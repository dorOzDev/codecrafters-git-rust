mod commands;

use std::env;
use std::io;

fn main() -> io::Result<()> {
    eprintln!("Logs from your program will appear here!");
    let args: Vec<String> = env::args().skip(1).collect(); // skip program name
    commands::run(&args)
}
