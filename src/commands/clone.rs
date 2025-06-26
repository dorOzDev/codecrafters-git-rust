use std::{io::{self}};
use crate::clone::{args::parse_args, http::fetch_refs, refs::parse_ref_advertisement};


pub fn run(args: &[String]) -> io::Result<()> {
    let clone_args = parse_args(&args)?;
    let refs_bytes = fetch_refs(&clone_args.url)?;
    println!("refs bytes len:{}", refs_bytes.len());
    parse_ref_advertisement(&refs_bytes)?;
    println!("Running git-clone with args: {},{}", clone_args.url, clone_args.target_dir.display());
    Ok(())
}
