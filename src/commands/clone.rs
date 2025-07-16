use std::{fs, io::{self}, path::Path, sync::Arc, sync::atomic::{AtomicBool, Ordering}};

use crate::{clone::{args::parse_args, packet_line::{pkt_line_unpacker::unpack_pkt_res, pkt_negotiator::negogiate_want}, refs::parse_ref_advertisement, transport::http::fetch_refs}, commands::init::init_git_dir, hash::{GitHash, HASH_HEX_LENGTH}};


pub fn run(args: &[String]) -> io::Result<()> { 
    let clone_args = parse_args(&args)?;
    validate_target_dir_empty_or_missing(&clone_args.target_dir)?;
    println!("Running git-clone with args: {},{}", clone_args.url, clone_args.target_dir.display());
    // Setup Ctrl+C handler to clean up target dir on interrupt
    setup_interrupt_cleanup(&clone_args.target_dir);

    let refs_bytes = fetch_refs(&clone_args.url)?;
    let refs = parse_ref_advertisement(&refs_bytes)?;
    let res = negogiate_want(&refs, &clone_args.url)?;
    if !clone_args.target_dir.exists() {
        fs::create_dir_all(&clone_args.target_dir)?;
    }
    init_git_dir(&clone_args.target_dir)?;
    // Steps after git dir creation: if any fails, delete the dir and return the error
    run_with_cleanup(|| refs.write_packed_refs(&clone_args.target_dir), &clone_args.target_dir)?;
    run_with_cleanup(|| unpack_pkt_res(res, &clone_args.target_dir), &clone_args.target_dir)?;

    // Remove Ctrl+C handler (not strictly necessary, but for clarity)
    // (ctrlc crate handlers are global and static, so this is just a comment)

    Ok(())
}

fn setup_interrupt_cleanup(dir: &Path) {

    let cleanup_dir = dir.to_path_buf();
    ctrlc::set_handler(move || {
        let _ = fs::remove_dir_all(&cleanup_dir);
        eprintln!("\nInterrupted. Cleaned up directory: {}", cleanup_dir.display());
        std::process::exit(130); // 130 is standard for SIGINT
    }).expect("Error setting Ctrl-C handler");
}

fn run_with_cleanup<F>(f: F, dir: &Path) -> io::Result<()>
    where
    F: FnOnce() -> io::Result<()>,
{
    if let Err(e) = f() {
        let _ = fs::remove_dir_all(dir);
        return Err(e);
    }
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