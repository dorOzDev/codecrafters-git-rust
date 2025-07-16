use std::{fs, io::{self}, path::Path};

use crate::{clone::{args::parse_args, packet_line::{pkt_line_unpacker::unpack_pkt_res, pkt_negotiator::negogiate_want}, refs::parse_ref_advertisement, transport::http::fetch_refs}, commands::init::init_git_dir,  utils::signal::{register_signal_handler, EventName}};


pub fn run(args: &[String]) -> io::Result<()> {
    let clone_args = parse_args(&args)?;
    validate_target_dir_empty_or_missing(&clone_args.target_dir)?;
    let target_dir = clone_args.target_dir.clone();
    println!("Running git-clone with args: {},{}", clone_args.url, clone_args.target_dir.display());
    register_signal_handler(EventName::CtrlC, move || {
        setup_interrupt_cleanup(&target_dir);
    });
    let refs_bytes = fetch_refs(&clone_args.url)?;
    let refs = parse_ref_advertisement(&refs_bytes)?;
    let res = negogiate_want(&refs, &clone_args.url)?;
    if !clone_args.target_dir.exists() {
        fs::create_dir_all(&clone_args.target_dir)?;
    }
    init_git_dir(&clone_args.target_dir)?;
    run_with_cleanup(|| refs.write_packed_refs(&clone_args.target_dir), &clone_args.target_dir)?;
    run_with_cleanup(|| unpack_pkt_res(res, &clone_args.target_dir), &clone_args.target_dir)?;

    Ok(())
}

fn setup_interrupt_cleanup(dir: &Path) {
    let cleanup_dir = dir.to_path_buf();
    let _ = fs::remove_dir_all(&cleanup_dir);
    eprintln!("\nInterrupted. Cleaned up directory: {}", cleanup_dir.display());
    std::process::exit(130); // 130 is standard for SIGINT
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