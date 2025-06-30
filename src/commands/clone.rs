use std::io;
use crate::clone::{args::parse_args, refs::parse_ref_advertisement, transport::{http::fetch_refs, pck_negotiator::run_upload_pck}};


pub fn run(args: &[String]) -> io::Result<()> { 
    let clone_args = parse_args(&args)?;
    println!("Running git-clone with args: {},{}", clone_args.url, clone_args.target_dir.display());
    let refs_bytes = fetch_refs(&clone_args.url)?;
    let refs = parse_ref_advertisement(&refs_bytes)?;

    run_upload_pck(&refs, &clone_args.url)?;

    Ok(())
}
