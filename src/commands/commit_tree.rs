use std::{io, iter::Peekable, slice::Iter};
use crate::{hash::GitHash, objects::{commit_object::{process_commit, Commit}, Person}};

pub fn run(args: &[String]) -> io::Result<()> {
    let commit = parse_commit(args)?;
    let hash = process_commit(&commit)?;
    println!("{}", hash.to_hex());
    Ok(())
}

fn parse_commit(args: &[String]) -> io::Result<Commit> {
    use std::io::{Error, ErrorKind};

    if args.is_empty() {
        return Err(Error::new(ErrorKind::InvalidInput, "usage: commit-tree <tree_hash> [-p <parent>]... -m <message>"));
    }

    let mut iter = args.iter().peekable();
    let tree_hash = parse_hash(&mut iter)?;
    log::debug!("parsed tree hash");
    let mut parent: Option<GitHash> = None;
    let mut message: Option<String> = None;

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-p" => {
                parent = Some(parse_hash(&mut iter)?);
            }
            "-m" => {
                message = Some(parse_commit_message(&mut iter));
            }
            _ => {
                log::debug!("unknown flag {}, ignoring it", arg);
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


fn parse_hash(iter: &mut Peekable<Iter<String>>) -> io::Result<GitHash> {
    if let Some(hex) = iter.next() {
        GitHash::from_hex(hex)
    } else {
        Err(io::Error::new(io::ErrorKind::InvalidInput, "missing value after hash flag"))
    }
}

fn parse_commit_message(iter: &mut Peekable<Iter<String>>) -> String {
    let mut msg_parts = Vec::new();

    while let Some(part) = iter.next() {
        if part.starts_with('-') {
            break;
        }
        msg_parts.push(part.clone());
    }

    let mut msg = msg_parts.join(" ");
    if !msg.ends_with('\n') {
        msg.push('\n');
    }

    msg
}

