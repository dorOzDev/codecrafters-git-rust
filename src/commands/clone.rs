use std::{fs, io::{self}, path::Path};

use crate::{clone::{args::parse_args, packet_line::{pkt_line_unpacker::unpack_pkt_res, pkt_negotiator::negogiate_want}, refs::parse_ref_advertisement, transport::http::fetch_refs}, commands::init::init_git_dir, hash::{GitHash, HASH_HEX_LENGTH}};


pub fn run(args: &[String]) -> io::Result<()> { 
    let clone_args = parse_args(&args)?;
    validate_target_dir_empty_or_missing(&clone_args.target_dir)?;
    println!("Running git-clone with args: {},{}", clone_args.url, clone_args.target_dir.display());
    let refs_bytes = fetch_refs(&clone_args.url)?;
    let refs = parse_ref_advertisement(&refs_bytes)?;
    let res = negogiate_want(&refs, &clone_args.url)?;
    if !clone_args.target_dir.exists() {
        fs::create_dir_all(&clone_args.target_dir)?;
    }
    init_git_dir(&clone_args.target_dir)?;
    unpack_pkt_res(res)?;
    Ok(())
}

pub fn validate_target_dir_empty_or_missing(target: &Path) -> io::Result<()> {
    if target.exists() {
        if !target.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!(
                    "destination path '{}' exists and is not a directory",
                    target.display()
                ),
            ));
        }

        if fs::read_dir(target)?.next().is_some() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!(
                    "destination path '{}' already exists and is not an empty directory",
                    target.display()
                ),
            ));
        }
    }

    Ok(())
}