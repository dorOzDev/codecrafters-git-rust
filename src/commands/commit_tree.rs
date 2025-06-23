use std::io;
use crate::{hash::GitHash, objects::{commit_object::{process_commit, Commit}, Person}};

pub fn run(args: &[String]) -> io::Result<()> {
    let commit = parse_commit(args)?;
    let hash = process_commit(&commit)?;
    println!("{}", hash);
    Ok(())
}

fn parse_commit(args: &[String]) -> io::Result<Commit> {
    use std::io::{Error, ErrorKind};

    if args.is_empty() {
        return Err(Error::new(ErrorKind::InvalidInput, "usage: commit-tree <tree_hash> [-p <parent>]... -m <message>"));
    }

    let mut iter = args.iter().peekable();

    let tree_hash_hex = iter.next().unwrap();
    let tree_hash = GitHash::from_hex(&tree_hash_hex)?;

    let mut parent: Option<GitHash> = None;
    let mut message: Option<String> = None;

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-p" => {
                if let Some(parent_hex) = iter.next() {
                    parent = Some(GitHash::from_hex(parent_hex)?);
                } else {
                    return Err(Error::new(ErrorKind::InvalidInput, "missing value after -p"));
                }
            }
            "-m" => {
                if let Some(msg) = iter.next() {
                    message = Some(msg.clone());
                } else {
                    return Err(Error::new(ErrorKind::InvalidInput, "missing commit message after -m"));
                }
            }
            _ => {
                log::debug!("unknown flag {}, ignoring it", &arg);
            }
        }
    }

    if message.is_none() {
        return Err(Error::new(ErrorKind::InvalidInput, "missing -m <message>"));
    }

    // TODO switch that when we have auth mechanism
    let timestamp = chrono::Utc::now().timestamp();
    let timezone = "+0000".to_string();
    let author = Person {
        name: "Dor Ozery".to_string(),
        email: "dor@example.com".to_string(),
        timestamp,
        timezone: timezone.clone(),
    };

    Ok(Commit {
        tree: tree_hash,
        parent_tree: parent,
        author: author.clone(),
        committer: author,
        message: message.unwrap(),
    })
}
