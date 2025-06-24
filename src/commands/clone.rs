use std::{io::{self, Error}, path::PathBuf};



pub fn run(args: &[String]) -> io::Result<()> {
    let clone_args = parse_args(&args)?;

    println!("Running git-clone with args: {:?}", args);
    Ok(())
}

pub struct CloneArgs {
    pub url: String,
    pub target_dir: PathBuf,
}

fn parse_args(args: &[String]) -> io::Result<CloneArgs> {
    if args.is_empty() {
        return Err(Error::new(io::ErrorKind::InvalidInput, "Usage: git clone <url> [directory]"));
    }

    let url = args[0].clone();
    let target_dir = if args.len() >= 2 {
        PathBuf::from(&args[1])
    } else {
        let trimmed = url.trim_end_matches(".git");
        let name = trimmed
            .rsplit('/')
            .find(|s| !s.is_empty())
            .unwrap_or("repo");
        PathBuf::from(name)
    };

    Ok(CloneArgs { url, target_dir })
}